// Rust 1.89.0 is giving false positive on needless lifetimes.
#![allow(clippy::needless_lifetimes)]

use crate::parsing::diagnostics::{Diagnostics, Span, code};
use crate::pitch::{Factor, Pitch};
use crate::to_anyhow;
use anyhow::anyhow;
use num_rational::Ratio;
use std::env;
use std::fmt::{Debug, Display};
use std::ops::Range;
use std::sync::LazyLock;
use winnow::combinator::{alt, opt, peek, preceded, separated};
use winnow::error::{ContextError, StrContext};
use winnow::stream::{AsChar, Offset};
use winnow::token::{any, one_of, take, take_until, take_while};
use winnow::{LocatingSlice, Parser};

mod parsers;

type CErr = ContextError<StrContext>;
type Inp<'s> = LocatingSlice<&'s str>;

#[derive(Debug, Clone)]
pub struct Spanned<'s, T: Debug> {
    span: Span,
    data: &'s str,
    t: T,
}

#[derive(Debug, Clone)]
pub enum LowToken {
    // Space, comments
    Space,
    Newline,
    Comment,
    // Punctuation
    Punctuation,
    // Things that literally map to the input
    Identifier,
    // Tokens that only appear at lower-level scanning
    RawNumber,
    RawString,
}

#[derive(Debug, Clone)]
/// Represents a pitch. All ratios also parse into pitches. If something that parsed into
/// a pitch was originally specified as a ratio, you can get the value as a ratio. During the
/// lexical phase, we never know whether a ratio is supposed to be a ratio or a pitch. During the
/// semantic phase, if a pitch was provided when a ratio was wanted, we can give an error at that
/// time.
pub enum PitchOrRatio {
    Ratio((Ratio<u32>, Pitch)),
    Pitch(Pitch),
}
impl PitchOrRatio {
    pub fn into_pitch(self) -> Pitch {
        match self {
            PitchOrRatio::Ratio((_, p)) => p,
            PitchOrRatio::Pitch(p) => p,
        }
    }

    pub fn try_into_ratio(self) -> Option<Ratio<u32>> {
        match self {
            PitchOrRatio::Ratio((r, _)) => Some(r),
            PitchOrRatio::Pitch(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    // Space, comments
    Space,
    Newline,
    Comment,
    // Punctuation
    Punctuation(char),
    // Things that literally map to the input
    Identifier,
    // Higher-level
    String(String),
    PitchOrRatio(PitchOrRatio),
}

fn is_space(x: char) -> bool {
    AsChar::is_space(x) || x == '\r'
}

fn make_spanned<'s, I, T: Debug>(
    input: &'s str,
    t: T,
) -> impl FnOnce((I, Range<usize>)) -> Spanned<'s, T> {
    move |(_, span)| Spanned {
        data: &input[span.clone()],
        span: span.into(),
        t,
    }
}

fn trace(msg: impl Display) {
    static TRACING: LazyLock<bool> = LazyLock::new(|| env::var("SYNTONIQ_TRACE_LEXER").is_ok());
    if *TRACING {
        eprintln!("{msg}");
    }
}

pub fn lex_pass1<'s>(src: &'s str) -> Result<Vec<Spanned<'s, LowToken>>, Diagnostics> {
    let diags = Diagnostics::new();
    let mut input = LocatingSlice::new(src);
    let start = input;
    let mut out: Vec<Spanned<LowToken>> = Vec::new();

    macro_rules! parse_as {
        ($parser: expr, $tok: expr) => {
            $parser
                .with_span()
                .parse_next(&mut input)
                .map(make_spanned(src, $tok))
        };
    }

    macro_rules! grab_char_as {
        ($tok: expr) => {
            parse_as!(any, $tok)
        };
    }

    while !input.is_empty() {
        let ch = input.chars().next().unwrap();
        let offset = input.offset_from(&start);
        let tok: Result<Spanned<LowToken>, CErr> = match ch {
            // Keep in same order as Token if possible, except when parsing order is significant.
            // That means check specific characters before character classes.
            '\n' => grab_char_as!(LowToken::Newline),
            ';' => parse_as!(preceded(';', take_until(0.., '\n')), LowToken::Comment),
            '"' => parse_as!(parsers::string_literal(&diags), LowToken::RawString),
            x if is_space(x) => parse_as!(take_while(0.., is_space), LowToken::Space),
            x if AsChar::is_dec_digit(x) => {
                parse_as!(parsers::raw_number(&diags), LowToken::RawNumber)
            }
            x if x.is_ascii_punctuation() => grab_char_as!(LowToken::Punctuation),
            x if AsChar::is_alpha(x) => parse_as!(parsers::identifier(), LowToken::Identifier),
            _ => {
                // discard token
                _ = any::<_, CErr>.parse_next(&mut input);
                diags.err(code::LEXICAL, offset..offset + 1, "unknown character");
                continue;
            }
        };
        match tok {
            Ok(t) => {
                trace(format!("lex pass 1: {t:?}"));
                out.push(t)
            }
            Err(e) => diags.err(
                code::LEXICAL,
                offset..offset + 1,
                format!("unknown lexical error: {e}"),
            ),
        }
        if input.offset_from(&start) == offset {
            // Consume a single character to prevent infinite loop
            _ = any::<_, CErr>(&mut input);
        }
    }
    if diags.has_errors() {
        Err(diags)
    } else {
        Ok(out)
    }
}

pub fn lex<'s>(src: &'s str) -> Result<Vec<Spanned<'s, Token>>, Diagnostics> {
    enum LexState {
        Top,
        _PartLine,
        _NoteLine,
    }
    let mut state = LexState::Top;

    let low_tokens = lex_pass1(src)?;
    let diags = Diagnostics::new();
    let mut input = low_tokens.as_slice();
    let mut out: Vec<Spanned<Token>> = Vec::new();

    let mut next_at_bol = true; // whether next token as at beginning of line
    while !input.is_empty() {
        let tok = &input[0];
        let at_bol = next_at_bol;
        // See if the next token will still be at the beginning of a line.
        next_at_bol = matches!(&tok.t, LowToken::Newline)
            || (next_at_bol && matches!(&tok.t, LowToken::Space));
        if at_bol && !next_at_bol {
            // This is the first non-blank token of the line. Determine state.
            let input_str = &src[tok.span.start..];
            state = if input_str.starts_with('[') {
                // TODO: commit to part or note
                LexState::_PartLine
            } else {
                LexState::Top
            }
        }
        match state {
            LexState::Top => handle_top_token(src, &mut input, &diags, &mut out),
            LexState::_PartLine => todo!(),
            LexState::_NoteLine => todo!(),
        }
    }
    if diags.has_errors() {
        Err(diags)
    } else {
        Ok(out)
    }
}

fn consume_one<T>(items: &mut &[T]) {
    if !items.is_empty() {
        *items = &items[1..]
    }
}

// TODO: needed?
fn _consume_until<T: Debug, F: Fn(&T) -> bool>(items: &mut &[T], pred: F) {
    while !items.is_empty() && !pred(&items[0]) {
        consume_one(items)
    }
}

fn promote<'s>(lt: &Spanned<'s, LowToken>, t: Token) -> Spanned<'s, Token> {
    Spanned {
        span: lt.span,
        data: lt.data,
        t,
    }
}

fn promote_and_consume_first<'s>(
    input: &mut &[Spanned<'s, LowToken>],
    t: Token,
) -> Spanned<'s, Token> {
    let tok = promote(&input[0], t);
    consume_one(input);
    tok
}

fn merge_span<T: Debug>(tokens: &[Spanned<T>]) -> Span {
    if tokens.is_empty() {
        0..1
    } else {
        tokens[0].span.start..tokens[tokens.len() - 1].span.end
    }
    .into()
}

/// Handle the current token, advancing input and appending to out as needed.
fn handle_top_token<'s>(
    src: &'s str,
    input: &mut &[Spanned<'s, LowToken>],
    diags: &Diagnostics,
    out: &mut Vec<Spanned<'s, Token>>,
) {
    // Look at the token. Some tokens can be immediately handled. Others indicate a branch
    // for further processing.
    let mut parse_pitch = false;
    let tok = &input[0];
    // At the top level, all we're allowed to have is directives, which consist of identifiers,
    // numbers, pitches, strings, and the syntactic punctation for identifiers. Anything else
    // (other than spaces and comments) is an error.
    let out_tok = match &tok.t {
        LowToken::RawString => {
            // Convert to an owned without delimiters and with quoted characters resolved.
            if tok.data.len() < 2 {
                // We already know this string starts and ends with `"`.
                unreachable!("length of raw string token < 2");
            }
            let data = &tok.data[1..tok.data.len() - 1];
            let s: String = data.chars().filter(|c| *c != '\\').collect();
            // We can just manually copy the token and advance input rather than using a parser.
            Some(promote_and_consume_first(input, Token::String(s)))
        }
        LowToken::RawNumber => {
            parse_pitch = true;
            None
        }
        LowToken::Space => Some(promote_and_consume_first(input, Token::Space)),
        LowToken::Newline => Some(promote_and_consume_first(input, Token::Newline)),
        LowToken::Comment => Some(promote_and_consume_first(input, Token::Comment)),
        LowToken::Punctuation => {
            let ch = tok.data.chars().next().expect("empty punctuation token");
            if "*-^".contains(ch) {
                parse_pitch = true;
                None
            } else if "(),=".contains(ch) {
                // This character part of calling a directive or valid in a pitch or number.
                Some(promote_and_consume_first(input, Token::Punctuation(ch)))
            } else {
                None
            }
        }
        LowToken::Identifier => Some(promote_and_consume_first(input, Token::Identifier)),
    };
    if let Some(tok) = out_tok {
        trace(format!("lex pass 2: {tok:?}"));
        out.push(tok);
        return;
    }
    let offset = tok.span.start;
    if !parse_pitch {
        diags.err(code::SYNTAX, offset..offset + 1, "unexpected character");
        consume_one(input);
        return;
    }

    if let Ok((pr, tokens)) = parsers::pitch_or_ratio(diags)
        .with_taken()
        .parse_next(input)
    {
        let span = merge_span(tokens);
        let t = Token::PitchOrRatio(pr);
        let data = &src[span];
        out.push(Spanned { span, data, t })
    } else {
        diags.err(code::SYNTAX, offset..offset + 1, "unable to parse as pitch");
        consume_one(input);
    }
}

/// Helper function for the Pitch struct
pub fn parse_pitch(s: &str) -> anyhow::Result<Pitch> {
    let mut p: Option<Pitch> = None;
    let mut diags: Option<Diagnostics> = None;
    match lex(s) {
        Ok(tokens) => {
            if tokens.len() == 1 {
                p = match tokens.into_iter().next().unwrap().t {
                    Token::PitchOrRatio(pr) => Some(pr.into_pitch()),
                    _ => None,
                };
            }
        }
        Err(d) => diags = Some(d),
    };
    if let Some(p) = p {
        return Ok(p);
    }
    let err = if let Some(diags) = diags
        && diags.has_errors()
    {
        to_anyhow(diags)
    } else {
        anyhow!("unable to parse pitch")
    };
    Err(anyhow!("{s}: {err}"))
}
