// This file contains the first pass of parsing from the raw input string to Vec<Token1>.
// These are low-level tokens that are handled by pass 2.

use crate::parsing::model;
use crate::parsing::model::{Diagnostics, Span, Spanned, Token, code};
use winnow::combinator::{alt, delimited, fail, preceded};
use winnow::error::{ContextError, StrContext};
use winnow::stream::{AsChar, Offset};
use winnow::token::take;
use winnow::token::{any, take_until, take_while};
use winnow::{LocatingSlice, Parser};

pub type CErr = ContextError<StrContext>;
pub type Input1<'s> = LocatingSlice<&'s str>;
pub type Token1<'s> = Spanned<Token<'s, Pass1>>;
trait Parser1Intermediate<'s, O>: Parser<Input1<'s>, O, CErr> {}
impl<'s, O, P: Parser<Input1<'s>, O, CErr>> Parser1Intermediate<'s, O> for P {}
trait Parser1<'s>: Parser1Intermediate<'s, Token1<'s>> {}
impl<'s, P: Parser1Intermediate<'s, Token1<'s>>> Parser1<'s> for P {}

/// Characters that have special meaning in note syntax and are not part of note names
static NOTE_PUNCTUATION: &str = "|/.:>~,'";
/// Characters allowed note names in addition to alphanumeric. This includes many punctuation
/// characters so pitches can be used in note names as well as making several characters available
/// for accidentals. We explicitly avoid characters that are syntactically ambiguous, like brackets
/// and parentheses, anything in NOTE_PUNCTUATION (which appear before or after note names), or
/// characters used in dynamics. This helps with parsing and also makes scores less visually
/// ambiguous. Avoid $ in case we introduce macros. Removing characters from this list breaks
/// backward compatibility, so we want to be cautious about over-doing it. It would be nice to
/// include `\`, but since this is a quoting character in strings and is hard to type cleanly
/// in TOML files, which is probably where we define note names, we're omitting it for now.
static NOTE_NAME_CHARACTERS: &str = "_*^/|+-!#%&";
/// Characters allowed in dynamics
static DYNAMIC_PUNCTUATION: &str = "|<>@";

#[derive(Debug, Clone, Copy)]
pub enum Pass1 {
    Unknown,
    Space,
    Newline,
    Comment,
    Punctuation,
    Identifier,
    Number { n: Spanned<u32> },
    String { inner_span: Span },
    NoteLeader { name_span: Span, note: Spanned<u32> },
    DynamicLeader { name_span: Span },
    NoteOptions { inner_span: Span },
    NoteName,
}
impl Pass1 {
    pub fn is_number(t: Token1) -> bool {
        matches!(t.value.t, Pass1::Number { .. })
    }

    pub fn get_number(t: &Token1) -> Option<u32> {
        match t.value.t {
            Pass1::Number { n } => Some(n.value),
            _ => None,
        }
    }

    pub fn is_string(t: Token1) -> bool {
        matches!(t.value.t, Pass1::String { .. })
    }

    pub fn get_string(t: &Token1) -> Option<Spanned<String>> {
        match t.value.t {
            Pass1::String { inner_span } => {
                let mut keep = true;
                let s: String = t.value.raw[inner_span.relative_to(t.span)]
                    .chars()
                    .filter(|c| {
                        // Skip any backslash not preceded by a backslash.
                        keep = !keep || *c != '\\';
                        keep
                    })
                    .collect();
                Some(Spanned::new(inner_span, s))
            }
            _ => None,
        }
    }

    pub fn is_note_leader(t: Token1) -> bool {
        matches!(t.value.t, Pass1::NoteLeader { .. })
    }

    pub fn get_note_leader<'s>(t: &'s Token1) -> Option<(Span, Spanned<u32>)> {
        match t.value.t {
            Pass1::NoteLeader { name_span, note } => Some((name_span, note.to_owned())),
            _ => None,
        }
    }

    pub fn is_dynamic_leader(t: Token1) -> bool {
        matches!(t.value.t, Pass1::DynamicLeader { .. })
    }

    pub fn get_dynamic_leader(t: &Token1) -> Option<Span> {
        match t.value.t {
            Pass1::DynamicLeader { name_span } => Some(name_span),
            _ => None,
        }
    }

    pub fn is_note_options(t: Token1) -> bool {
        matches!(t.value.t, Pass1::NoteOptions { .. })
    }

    pub fn get_note_options(t: &Token1) -> Option<Span> {
        match t.value.t {
            Pass1::NoteOptions { inner_span } => Some(inner_span),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum LexState {
    Top,
    DynamicLine,
    NoteLine,
}

fn parse1_intermediate<'s, P, F, T, O>(p: P, f: F) -> impl Parser1Intermediate<'s, O>
where
    O: 's,
    P: Parser1Intermediate<'s, T>,
    F: Fn(&'s str, Span, T) -> O,
{
    p.with_span().with_taken().map(move |((out, span), raw)| {
        let span: Span = span.into();
        f(raw, span, out)
    })
}

fn is_space(x: char) -> bool {
    AsChar::is_space(x) || x == '\r'
}

fn newline<'s>() -> impl Parser1<'s> {
    parse1_token(any, move |_raw, _span, _out| Pass1::Newline)
}

fn punctuation<'s>() -> impl Parser1<'s> {
    parse1_token(any, move |_raw, _span, _out| Pass1::Punctuation)
}

fn unknown<'s>() -> impl Parser1<'s> {
    parse1_token(any, move |_raw, _span, _out| Pass1::Unknown)
}

fn parse1_token<'s, P, F, T>(p: P, f: F) -> impl Parser1<'s>
where
    P: Parser1Intermediate<'s, T>,
    F: Fn(&'s str, Span, T) -> Pass1,
{
    parse1_intermediate(p, move |raw, span, out| {
        Token::new_spanned(raw, span, f(raw, span, out))
    })
}

fn comment<'s>() -> impl Parser1<'s> {
    parse1_token(preceded(';', take_until(0.., '\n')), |_raw, _span, _out| {
        Pass1::Comment
    })
}

fn space<'s>() -> impl Parser1<'s> {
    parse1_token(take_while(0.., is_space), |_raw, _span, _out| Pass1::Space)
}

fn identifier<'s>() -> impl Parser1<'s> {
    parse1_token(
        (
            take_while(1, |c: char| AsChar::is_alpha(c)),
            take_while(0.., |c: char| AsChar::is_alphanum(c) || c == '_'),
        ),
        |_raw, _span, _out| Pass1::Identifier,
    )
}

fn number_intermediate<'s>(diags: &Diagnostics) -> impl Parser1Intermediate<'s, Spanned<u32>> {
    parse1_intermediate(take_while(1.., AsChar::is_dec_digit), |_raw, span, out| {
        // Report error after consuming all the digits. Make sure this can parse to
        // an i32. We know it's positive, so that means it will also parse to a u32.
        // Other code unwraps parse calls on this data.
        let n = match out.parse::<u32>() {
            Ok(n) => n,
            Err(e) => {
                diags.err(code::LEXICAL, span, format!("while parsing number: {e}"));
                1
            }
        };
        Spanned::new(span, n)
    })
}

fn number<'s>(diags: &Diagnostics) -> impl Parser1<'s> {
    parse1_token(number_intermediate(diags), |_raw, _span, out| {
        Pass1::Number { n: out }
    })
}

fn string_literal<'s>(diags: &Diagnostics) -> impl Parser1<'s> {
    fn inner<'s>(input: &mut Input1<'s>) -> winnow::Result<&'s str> {
        "\"".parse_next(input)?;
        let start = *input;
        loop {
            if input.starts_with('\\') {
                take(2usize).parse_next(input)?;
            } else if input.starts_with('"') {
                any.parse_next(input)?;
                break Ok(&start[..input.offset_from(&start) - 1]);
            } else {
                any.parse_next(input)?;
            }
        }
    }
    parse1_token(inner, |_raw, span, out| {
        let mut chars = out.chars();
        let mut offset = span.start + 1;
        while let Some(ch) = chars.next() {
            if ch == '\\'
                && let Some(next) = chars.next()
            {
                let char_len = next.len_utf8();
                if !['\\', '"'].contains(&next) {
                    // Span is character after the backslash
                    diags.err(
                        code::LEXICAL,
                        offset + 1..offset + 1 + char_len,
                        "invalid quoted character",
                    );
                }
                // Skip the character after the backslash; below will skip the backslash
                offset += char_len;
            } else if ['\r', '\n'].contains(&ch) {
                diags.err(
                    code::LEXICAL,
                    offset..offset + 1,
                    "string may not contain newline characters",
                );
            }
            offset += ch.len_utf8();
        }
        Pass1::String {
            inner_span: (span.start + 1..span.end - 1).into(),
        }
    })
}

fn note_leader<'s>(diags: &Diagnostics) -> impl Parser1<'s> {
    parse1_token(
        delimited(
            '[',
            (
                identifier().with_span().map(|(_, span)| Span::from(span)),
                preceded('.', number_intermediate(diags)),
            ),
            ']',
        ),
        |_raw, _span, (name_span, note)| Pass1::NoteLeader { name_span, note },
    )
}

fn dynamic_leader<'s>() -> impl Parser1<'s> {
    parse1_token(
        delimited(
            '[',
            identifier().with_span().map(|(_, span)| Span::from(span)),
            ']',
        ),
        |_raw, _span, name_span| Pass1::DynamicLeader { name_span },
    )
}

fn note_options<'s>() -> impl Parser1<'s> {
    parse1_token(
        delimited('(', take_until(0.., ')').with_span(), ')'),
        |_raw, _span, (_, inner_span)| Pass1::NoteOptions {
            inner_span: inner_span.into(),
        },
    )
}

fn note_name<'s>() -> impl Parser1<'s> {
    parse1_token(
        (
            take_while(1, |c: char| AsChar::is_alpha(c)),
            take_while(0.., |c: char| {
                AsChar::is_alphanum(c) || NOTE_NAME_CHARACTERS.contains(c)
            }),
        ),
        |_raw, _span, _out| Pass1::NoteName,
    )
}

pub fn parse1<'s>(src: &'s str) -> Result<Vec<Token1<'s>>, Diagnostics> {
    let diags = Diagnostics::new();
    let mut input = LocatingSlice::new(src);
    let start = input;
    let mut out: Vec<Token1> = Vec::new();
    let mut state = LexState::Top;
    let mut next_at_bol = true; // whether next token as at beginning of line

    while !input.is_empty() {
        let ch = input.chars().next().unwrap();
        let offset = input.offset_from(&start);

        let at_bol = next_at_bol;
        // See if the next token will still be at the beginning of a line.
        next_at_bol = ch == '\n' || (next_at_bol && is_space(ch));
        if at_bol && !next_at_bol {
            // This is the first non-blank token of the line. Determine state.
            if let Ok((tok, new_state)) = alt((
                note_leader(&diags).map(|x| (x, LexState::NoteLine)),
                dynamic_leader().map(|x| (x, LexState::DynamicLine)),
                fail,
            ))
            .parse_next(&mut input)
            {
                state = new_state;
                model::trace(format!("lex pass 1: {tok:?} -> {state:?}"));
                out.push(tok);
                continue;
            } else {
                state = LexState::Top;
            }
        }

        macro_rules! parse_next {
            ($p:expr) => {
                $p.parse_next(&mut input)
            };
        }

        let tok = match ch {
            '\n' => parse_next!(newline()),
            ';' => parse_next!(comment()),
            x if is_space(x) => parse_next!(space()),
            x if AsChar::is_dec_digit(x) => parse_next!(number(&diags)),
            _ => match state {
                LexState::Top => match ch {
                    '"' => parse_next!(string_literal(&diags)),
                    x if x.is_ascii_punctuation() => parse_next!(punctuation()),
                    x if AsChar::is_alpha(x) => parse_next!(identifier()),
                    _ => parse_next!(unknown()),
                },
                LexState::NoteLine => match ch {
                    '(' => parse_next!(note_options()),
                    x if NOTE_PUNCTUATION.contains(x) => parse_next!(punctuation()),
                    x if AsChar::is_alpha(x) => parse_next!(note_name()),
                    _ => parse_next!(unknown()),
                },
                LexState::DynamicLine => match ch {
                    x if DYNAMIC_PUNCTUATION.contains(x) => parse_next!(punctuation()),
                    _ => parse_next!(unknown()),
                },
            },
        };
        match tok {
            Ok(t) => {
                if matches!(t.value.t, Pass1::Unknown) {
                    // discard token
                    diags.err(code::LEXICAL, t.span, "unknown character");
                } else {
                    model::trace(format!("lex pass 1: {t:?}"));
                    out.push(t)
                }
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

#[cfg(test)]
mod tests;
