// This file contains the first pass of parsing from the raw input string to Vec<Token1>.
// These are low-level tokens that are handled by pass 2.

use crate::parsing::model;
use crate::parsing::model::{Diagnostics, SpannedToken, code};
use winnow::combinator::preceded;
use winnow::error::{ContextError, StrContext};
use winnow::stream::{AsChar, Offset};
use winnow::token::take;
use winnow::token::{any, take_until, take_while};
use winnow::{LocatingSlice, Parser};

pub type CErr = ContextError<StrContext>;
pub type Inp<'s> = LocatingSlice<&'s str>;

#[derive(Debug, Clone)]
pub enum Token1 {
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

fn is_space(x: char) -> bool {
    AsChar::is_space(x) || x == '\r'
}

fn raw_identifier<'s>() -> impl Parser<Inp<'s>, &'s str, CErr> {
    (
        take_while(1, |c: char| AsChar::is_alpha(c)),
        take_while(0.., |c: char| AsChar::is_alphanum(c) || c == '_'),
    )
        .take()
        .context(StrContext::Label("identifier"))
}

fn raw_number<'s>(diags: &Diagnostics) -> impl FnMut(&mut Inp<'s>) -> winnow::Result<&'s str> {
    move |input| {
        take_while(1.., AsChar::is_dec_digit)
            .with_span()
            .parse_next(input)
            .map(|(s, span)| {
                // Report error after consuming all the digits. Make sure this can parse to
                // an i32. We know it's positive, so that means it will also parse to a u32.
                // Other code unwraps parse calls on this data.
                if let Err(e) = s.parse::<i32>() {
                    diags.err(code::LEXICAL, span, format!("while parsing number: {e}"));
                }
                s
            })
    }
}

fn string_literal<'s>(diags: &Diagnostics) -> impl FnMut(&mut Inp<'s>) -> winnow::Result<&'s str> {
    fn inner<'s>(input: &mut Inp<'s>) -> winnow::Result<&'s str> {
        let start = *input;
        "\"".parse_next(input)?;
        loop {
            if input.starts_with('\\') {
                take(2usize).parse_next(input)?;
            } else if input.starts_with('"') {
                any.parse_next(input)?;
                break Ok(&start[..input.offset_from(&start)]);
            } else {
                any.parse_next(input)?;
            }
        }
    }
    move |input| {
        inner.with_span().parse_next(input).map(|(s, span)| {
            let mut chars = s.chars();
            let mut offset = span.start;
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
            s
        })
    }
}

pub fn parse1<'s>(src: &'s str) -> Result<Vec<SpannedToken<'s, Token1>>, Diagnostics> {
    let diags = Diagnostics::new();
    let mut input = LocatingSlice::new(src);
    let start = input;
    let mut out: Vec<SpannedToken<Token1>> = Vec::new();

    macro_rules! parse_as {
        ($parser: expr, $tok: expr) => {
            $parser
                .with_span()
                .parse_next(&mut input)
                .map(model::make_spanned(src, $tok))
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
        let tok: Result<SpannedToken<Token1>, CErr> = match ch {
            // Keep in same order as Token if possible, except when parsing order is significant.
            // That means check specific characters before character classes.
            '\n' => grab_char_as!(Token1::Newline),
            ';' => parse_as!(preceded(';', take_until(0.., '\n')), Token1::Comment),
            '"' => parse_as!(string_literal(&diags), Token1::RawString),
            x if is_space(x) => parse_as!(take_while(0.., is_space), Token1::Space),
            x if AsChar::is_dec_digit(x) => {
                parse_as!(raw_number(&diags), Token1::RawNumber)
            }
            x if x.is_ascii_punctuation() => grab_char_as!(Token1::Punctuation),
            x if AsChar::is_alpha(x) => parse_as!(raw_identifier(), Token1::Identifier),
            _ => {
                // discard token
                _ = any::<_, CErr>.parse_next(&mut input);
                diags.err(code::LEXICAL, offset..offset + 1, "unknown character");
                continue;
            }
        };
        match tok {
            Ok(t) => {
                model::trace(format!("lex pass 1: {t:?}"));
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

#[cfg(test)]
mod tests;
