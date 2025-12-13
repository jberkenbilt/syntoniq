// This file contains the first pass of parsing from the raw input string to Vec<Token1>.
// These are low-level tokens that are handled by pass 2.
//
// Start with the comments in ../parsing.rs. Then come back here.
//
// This file includes a narrative that you can follow to understand the parsing. It is intended to
// make this code understandable even if you are new to parser combinators (or have been away from
// them for a while), though reading through the winnow tutorial is highly recommended.
//
// Search for `Pass1 Step 1:` and follow the trail.

use crate::parsing::diagnostics::{Diagnostics, code};
use crate::parsing::model;
use crate::parsing::model::{Span, Spanned, Token};
use serde::Serialize;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use winnow::combinator::{alt, delimited, fail, preceded};
use winnow::error::{ContextError, StrContext};
use winnow::stream::{AsChar, Offset};
use winnow::token::take;
use winnow::token::{any, take_while};
use winnow::{LocatingSlice, Parser};

// Pass1 Step 4: define a bunch of helper types and traits. This gets into the guts of how winnow
// parsers work. The winnow `Parser<I, O, E>` (input, output, error) trait, among other things,
// requires parse methods such as `parse_next` that take a mutable input slice of type `I` and
// return a result of type `Result<O, E>`. The Parser trait is implemented in the crate for a bunch
// of things, including even characters and tuples of parsers. This is extremely flexible, but it
// can also make compiler error messages impenetrable if you make a small mistake. The `I`, `O`, and
// `E` types can be nearly anything, but you get certain extra crate-provided features if `I` is
// `&str`, and a few more beyond that if it is `LocatingSlice<&str>`. In particular, using
// `LocatingSlice` makes it possible for us to use `with_span()`, and some other things, like
// calling `map` directly on a parser, are available with `&str`. Note that the code sometimes calls
// regular rust standard library `map` on `Result` and sometimes calls `map` on the parser type. For
// backtracking to work, the error type should be `ContextError` We use `ContextError<StrContext>`,
// which is the most usual thing to use and allows certain automatic error generation to work,
// though nothing in the code actually depends on `StrContext` as the generic type for
// `ContextError`. Note that in all cases, we use the lifetime `'s` for the lifetime of the original
// input slice. This enables the tokens to contain slices of the input without making any copies.
pub type CErr = ContextError<StrContext>;
// To simplify the method definitions of our parsers and helpers, define types and traits for the
// things used by pass 1.
pub type Input1<'s> = LocatingSlice<&'s str>;
// All of our parsers work with our `Input1` type. Our "intermediate parsers" can return any type.
// These are parsers intended for use by other parsers. This rust idiom of creating a trait that has
// nothing but a supertrait and then globally implements itself for all instances of the supertrait
// is the trait equivalent of a type alias.
trait Parser1Intermediate<'s, O>: Parser<Input1<'s>, O, CErr> {}
impl<'s, O, P: Parser<Input1<'s>, O, CErr>> Parser1Intermediate<'s, O> for P {}
// Our ultimate pass-1 parsers return `Token1` which carries spanned payload of type `Pass1`. These
// are the basic token types we recognize in this pass. The `Spanned` type attaches a span, and the
// `Token` type attaches the raw input. Breaking it up this way makes it easier to promote data from
// pass 1 to pass 2 and to create generic combinators (in the general sense) that match on spanned
// things or on spanned tokens of any type.
pub type Token1<'s> = Spanned<Token<'s, Pass1>>;
// We define `Parser1` as a trait that parses from our basic input type to our output token type.
// Resume with pass 1 step 5.
trait Parser1<'s>: Parser1Intermediate<'s, Token1<'s>> {}
impl<'s, P: Parser1Intermediate<'s, Token1<'s>>> Parser1<'s> for P {}

/// Characters that have special meaning in note syntax and may appear separately from note names
static NOTE_PUNCTUATION: &str = "|/.:>~^,'";
/// Characters allowed note names in addition to alphanumeric. This includes many punctuation
/// characters so pitches can be used in note names as well as making several characters available
/// for accidentals. We explicitly avoid characters that are syntactically ambiguous, like brackets
/// and parentheses, anything in NOTE_PUNCTUATION (which appear before or after note names), or
/// characters used in dynamics. This helps with parsing and also makes scores less visually
/// ambiguous. Avoid $ in case we introduce macros. Removing characters from this list breaks
/// backward compatibility, so we want to be cautious about over-doing it. Avoid @ because of its
/// use in layouts and dynamics.
static NOTE_NAME_CHARACTERS: &str = "_*^/.|+-!\\#%&";
/// Characters allowed in dynamics
static DYNAMIC_PUNCTUATION: &str = "|<>@/.";
/// Characters allowed in definitions outside note names and numbers, including pitch characters,
/// anchor/place-holder characters for layouts, and octave markers.
static DEFINITION_PUNCTUATION: &str = "^*|/.@~,'";

#[derive(Serialize, Debug, Clone, Copy)]
/// The Pass1 type contains payload for pass-1 tokens. These contain only spans and numbers, which
/// makes it cheap to implement Copy. The implementation of Pass1 enables us to extract information
/// about of the raw data at a higher level for use by pass 2.
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
    NoteName,
    DefinitionStart,
    DefinitionEnd,
}
impl Display for Pass1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Pass1::Number { n } => write!(f, "Number:{n}"),
            Pass1::String { inner_span } => write!(f, "String:{inner_span}"),
            Pass1::NoteLeader { name_span, note } => write!(f, "NoteLeader:{name_span}/{note}"),
            Pass1::DynamicLeader { name_span } => write!(f, "DynamicLeader:{name_span}"),
            _ => write!(f, "{self:?}"),
        }
    }
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

    pub fn get_string<'s>(t: &Token1<'s>) -> Option<Spanned<Cow<'s, str>>> {
        match t.value.t {
            Pass1::String { inner_span } => {
                let inner = &t.value.raw[inner_span.relative_to(t.span)];
                if inner.find('\\').is_some() {
                    let mut keep = true;
                    let s: String = t.value.raw[inner_span.relative_to(t.span)]
                        .chars()
                        .filter(|c| {
                            // Skip any backslash not preceded by a backslash.
                            keep = !keep || *c != '\\';
                            keep
                        })
                        .collect();
                    Some(Spanned::new(inner_span, Cow::Owned(s)))
                } else {
                    Some(Spanned::new(inner_span, Cow::Borrowed(inner)))
                }
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
}

#[derive(Debug)]
enum LexState {
    Top,
    DynamicLine,
    NoteLine,
    Definition,
}

// Pass1 Step 5: This is the heart of our parsing logic. It's simple, but there's a lot to unpack.
// Our generic parameters are
// - P: the parser type, constrained to our intermediate parser
// - T: the type returned by the intermediate parser
// - F: a function that takes the raw data matched by the intermediate parser along with its span
//   and the parser's output and returns something of type O
// - O: the output type ultimately returned.
// This pattern allows us to encapsulate the logic of calling with_span and with_taken so that all
// intermediate parsing logic has access to the full captured input string and its span. This makes
// it much more ergonomic (with a lot less boilerplate) to write intermediate parsers that return
// arbitrary types.
fn parse1_intermediate<'s, P, T, F, O>(p: P, f: F) -> impl Parser1Intermediate<'s, O>
where
    O: 's,
    P: Parser1Intermediate<'s, T>,
    F: Fn(&'s str, Span, T) -> O,
{
    // `with_span` changes the output of P from T to (T, span). With_taken returns the output of
    // its predecessor from T to (T, consumed-input). The result is that we have ((T, span),
    // consumed-input). This just grabs these, re-orders them, and passes them to the provided
    // function. Continue with step 6.
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

fn unknown<'s>() -> impl Parser1Intermediate<'s, Span> {
    parse1_intermediate(any, move |_raw, span, _out| span)
}

// Pass1 Step 7: all our top-level parsers return Token1. This helper allows any intermediate
// parser's output to be mapped into a Pass1 token. It just takes the output of the intermediate
// parser and transforms it to a token along with the span and raw data. Proceed to step 8.
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
    parse1_token(
        // Use take_while rather than take_until so a comment can appear at the end of a file with
        // no trailing newline.
        preceded(';', take_while(0.., |ch| ch != '\n')),
        |_raw, _span, _out| Pass1::Comment,
    )
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
    // Pass1 Step 3: This is a function that recognizes a number that can be parsed into a `u32`.
    // First, look at step 4, which defines a bunch of helper types. You will be routed back here.
    // Pass1 Step 6: This is our first use of a native winnow parser: `take_while`. This combinator
    // consumes inputs as long as the predicate matches. This is an example of a parser matching
    // liberally and doing checks with our own diagnostics. In particular, the parser we pass to
    // parse1_intermediate consumes 1 or more consecutive digits, so regardless of the number of
    // digits, the parser will match and consume those digits. Then...
    parse1_intermediate(take_while(1.., AsChar::is_dec_digit), |_raw, span, out| {
        // ...once we have captured the digits, we use our own Diagnostics system to report an error
        // if this doesn't parse to an i32. While we return a `u32`, we ensure it can parse to an
        // i32 so we can safely negate it without running into overflow issues. This implementation
        // is very deliberate. winnow offers things like `verify` and `try_map` that allow us to
        // reject matched tokens. We could have implemented this to reject the tokens if they didn't
        // parse into a `u32`, but if we did, we'd end up with undesirable results. For example, we
        // parsed a 20-digit number, the rule would fail, which would cause later in the main loop
        // to consume just the first digit as an "unknown". Then we'd try on a 19-digit number,
        // which would do the same, consuming one digit at a time until we finally got something
        // that worked. This would give very unhelpful error messages. We could also limit the
        // number of digits we parsed, but then we'd parse a 20-digit number as two to three 9- or
        // 10-digit numbers and maybe a 1- or 2-digit number, not separated by space, which would
        // definitely violate the principle of least surprise. This is a long-winded explanation,
        // but it gets to the heart of how all the parsers are implemented. Continue with Pass1 Step
        // 7.
        let n = match out.parse::<u32>() {
            Ok(n) => n,
            Err(_) => {
                diags.err(code::NUM_RANGE, span, "number too large for 32 bits");
                1
            }
        };
        Spanned::new(span, n)
    })
}

fn number<'s>(diags: &Diagnostics) -> impl Parser1<'s> {
    // Pass1 Step 2: this is an example of a basic custom combinator. Following this path shows
    // the basic structure of all our combinators. Proceed to step 3 in number_intermediate. Pass1
    // Step 8: this function returns a Number token. We break it down into number_intermediate,
    // which returns a spanned u32, and the number token itself. This function trivially wraps the
    // spanned number in a token. The reason for the separation is that it allows other functions to
    // get at the number without having to do a match on a token and pull the number back out. See
    // the parsers for `ratio` and `exponent` for examples. This completes the tour of pass1
    // parsing. From here, you should be able to follow the code and comments in the rest of the
    // file. When read, proceed to Pass2 Step 1 in pass2.rs.
    parse1_token(number_intermediate(diags), |_raw, _span, out| {
        Pass1::Number { n: out }
    })
}

fn string_literal<'s>(diags: &Diagnostics) -> impl Parser1<'s> {
    // The `inner` function consumes characters starting and ending with `"` and unconditionally
    // taking any character immediately after `\`. It should only be called when we've already seen
    // a `"` character and have committed to reading a string. By consuming all characters and then
    // checking, we make it possible to consume strings and then report on errors in strings.
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
        // After capturing the characters, iterate through, perform additional validations, and
        // normalize the results into the internal span that includes the body of the string without
        // the quotes. See Pass1::get_string for the code that converts this to an owned string.
        // Storing only spans and copiable items in Pass1 enables us to keep the Token1 type
        // copiable.
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
                        code::STRING,
                        offset + 1..offset + 1 + char_len,
                        "invalid quoted character",
                    );
                }
                // Skip the character after the backslash; below will skip the backslash
                offset += char_len;
            } else if ['\r', '\n'].contains(&ch) {
                // Another approach would be to break out of the string and report an unterminated
                // string. In any case, this is a pass-1 error, so if this happens, it will severely
                // limit the types of errors we will see for the rest of the file. An actual
                // forgotten quote will generate spurious instances of this error message as well as
                // lots of invalid character errors for the characters that are supposed to be
                // inside subsequent strings. Hopefully the error text makes it clear enough.
                diags.err(
                    code::STRING,
                    offset..offset + 1,
                    "string may not contain newline characters; check for missing closed quote",
                );
            }
            offset += ch.len_utf8();
        }
        Pass1::String {
            inner_span: (span.start + 1..span.end - 1).into(),
        }
    })
}

fn definition_start<'s>() -> impl Parser1<'s> {
    parse1_token(take_while(2, '<'), |_raw, _span, _chars| {
        Pass1::DefinitionStart
    })
}

fn definition_end<'s>() -> impl Parser1<'s> {
    parse1_token(take_while(2, '>'), |_raw, _span, _chars| {
        Pass1::DefinitionEnd
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
    // Pass1 Step 1: this is where parsing starts. Each "pass" of parsing creates a Diagnostics
    // object. If it is empty at the end of parsing, all concerns by this pass have been met.
    // Otherwise, we return the Diagnostics as the "Error" type of the Result. This "Step 1"
    // description continues until we reference a future step, so just read comments in order
    // through the file until redirected.

    let diags = Diagnostics::new();
    // `LocatingSlice` is a winnow type that allows spans to be created. We have our own Span type,
    // but a span is just a half-open interval of byte offsets over the input. The input is a UTF-8
    // string, but all offsets are byte offsets. When skipping over characters, this code must keep
    // that in mind (by using len_utf8, for example). Translation of byte offsets into lines and
    // columns is delegated to the annotate_snippets crate, which does our error reporting.
    let mut input = LocatingSlice::new(src);
    // Keeping a slice that points to the beginning of `input` allows us (via the parsers) to
    // continue to advance `input` while giving us a relative point for computing the current
    // offset. This is more reliable than keeping a count.
    let start = input;
    let mut out: Vec<Token1> = Vec::new();
    // This pass is mostly concerned with simple lexing -- recognizing basic tokens without any
    // meaning, like numbers and strings. However, since we recognize different tokens depending on
    // whether we're at the top level, in a note line, or in a dynamic line (see the docs for a
    // description of the overall syntoniq syntax), we have a simple state machine that tracks which
    // set of tokens we are matching. This is one area where we have strictly more power than a
    // traditional regular-expression-based parser would have.
    let mut state = LexState::Top;
    let mut next_at_bol = true; // whether next token as at beginning of line

    macro_rules! parse_next {
        ($p:expr) => {
            $p.parse_next(&mut input).ok()
        };
    }

    // Keep consuming tokens until we've got them all.
    while !input.is_empty() {
        // Peek at the first character.
        let ch = input.chars().next().unwrap();
        let offset = input.offset_from(&start);

        // Note if we are currently at the beginning of a line, and see if the next token will still
        // be at the beginning of a line. This is effectively skipping over initial whitespace.
        let at_bol = next_at_bol;
        next_at_bol = ch == '\n' || (next_at_bol && is_space(ch));
        if ch == '\n' && matches!(state, LexState::DynamicLine | LexState::NoteLine) {
            state = LexState::Top;
        }
        let (tok, new_state) =
            if at_bol && !next_at_bol && matches!(state, LexState::Top) && ch == '[' {
                // This is the first non-blank token of the line. Determine state. This is our first
                // example of using parser combinators.  The `alt` combinator is a special parser that
                // takes a tuple of parsers that all return the same type. It tries them in order. If a
                // parser fails to match, it tries the next one. Ending with `fail`, which always fails,
                // prevents a partial success where some tokens are consumed. In this particular
                // instance, we are seeing whether the input matches a note leader or a dynamic leader.
                // If either of those are matched, keep the matching token and switch lexer states to
                // recognize the tokens that are valid in that context. If none match, revert to the
                // top-level state.
                if let Ok((tok, new_state)) = alt((
                    note_leader(&diags).map(|x| (x, LexState::NoteLine)),
                    dynamic_leader().map(|x| (x, LexState::DynamicLine)),
                    fail,
                ))
                .parse_next(&mut input)
                {
                    (Some(tok), new_state)
                } else {
                    (None, LexState::Top)
                }
            } else if input.starts_with("<<") && matches!(state, LexState::Top) {
                let tok = parse_next!(definition_start());
                (tok, tok.map(|_| LexState::Definition).unwrap_or(state))
            } else if ch == '>' && matches!(state, LexState::Definition) {
                (None, LexState::Top)
            } else {
                (None, state)
            };
        state = new_state;
        if let Some(tok) = tok {
            model::trace(format!("lex pass 1: {tok} -> {state:?}"));
            out.push(tok);
            continue;
        }

        // At this point, we are in a known state, so this is the main pass1 parser loop. In pass 1,
        // we are parsing at a level where we can always determine what kind of token to match based
        // on just the first character. Rather than using `peek` (which tries a parser but doesn't
        // consume the tokens), we can just look at the character directly.
        //
        // The parsers are attempted sequentially and will consume tokens if successful. We have the
        // freedom to have multiple matching rules, but it is up to us to make sure the most
        // specific rule is tried first. For example, we have to try specific punctuation characters
        // before a general catch-all punctuation rule.
        let tok = match ch {
            // In all contexts, comments, white space, and newlines look the same.
            '\n' => parse_next!(newline()),
            x if is_space(x) => parse_next!(space()),
            ';' => parse_next!(comment()),
            // If we found a `[` at the beginning of the line that matched a leader, it would have
            // been consumed, so this would happen if a `[` didn't match. That is worthy of its own
            // distinct error message. This is an example of where use of combinators with a
            // hand-coded state machine enables higher quality error messages.
            '[' if at_bol => {
                diags.err(
                    code::LINE_START,
                    offset..offset + 1,
                    "encountered '[' but didn't recognize note or dynamic line leader",
                );
                parse_next!(punctuation())
            }
            // Numbers are the same for all levels.
            x if AsChar::is_dec_digit(x) => parse_next!(number(&diags)),
            // From here, we branch into separate branches. Different kinds of tokens are matched
            // depending on the lexer state.
            _ => match state {
                LexState::Top => match ch {
                    '"' => parse_next!(string_literal(&diags)),
                    '>' if input.starts_with(">>") => parse_next!(definition_end()),
                    x if x.is_ascii_punctuation() => parse_next!(punctuation()),
                    x if AsChar::is_alpha(x) => parse_next!(identifier()),
                    _ => None,
                },
                LexState::NoteLine => match ch {
                    x if NOTE_PUNCTUATION.contains(x) => parse_next!(punctuation()),
                    x if AsChar::is_alpha(x) => parse_next!(note_name()),
                    _ => None,
                },
                LexState::DynamicLine => match ch {
                    x if DYNAMIC_PUNCTUATION.contains(x) => parse_next!(punctuation()),
                    _ => None,
                },
                LexState::Definition => match ch {
                    x if DEFINITION_PUNCTUATION.contains(x) => parse_next!(punctuation()),
                    x if AsChar::is_alpha(x) => parse_next!(note_name()),
                    _ => None,
                },
            },
        };
        match tok {
            Some(t) => {
                // If we got a token, push it onto the output and give a trace log for debugging.
                model::trace(format!("lex pass 1: {t}"));
                out.push(t)
            }
            None => {
                // Consume the token. Issuing a diagnostic error will prevent the accumulated tokens
                // from being returned. Pass 2 never needs to encounter Unknown tokens.
                let span = parse_next!(unknown()).unwrap();
                diags.err(code::SYNTAX, span, "this character is not allowed here");
            }
        }
    }
    if out.is_empty() {
        diags.err(code::EMPTY_FILE, 0..1, "this file is empty");
    }
    if diags.has_errors() {
        Err(diags)
    } else {
        Ok(out)
    }
    // Search for "Pass1 Step 2".
}

#[cfg(test)]
mod tests;
