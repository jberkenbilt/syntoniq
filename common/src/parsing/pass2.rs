// This file contains the second pass of parsing from the output of pass1. Read comments in
// ../parsing.rs.

use crate::parsing::diagnostics::{self, Diagnostics, code};
use crate::parsing::model::{
    DataBlock, Dynamic, DynamicLine, GetSpan, Hold, Identifier, LayoutBlock, LayoutItem,
    LayoutItemType, Note, NoteLeader, NoteLine, NoteModifier, NoteOctave, NoteOrIdentifier, Param,
    ParamValue, PitchOrNumber, RawDirective, RegularNote, ScaleBlock, ScaleNote, Span, Spanned,
    Token,
};
use crate::parsing::model::{DynamicChange, DynamicLeader, RegularDynamic};
use crate::parsing::pass1::{Pass1, Token1};
use crate::parsing::{model, pass1};
use crate::pitch::{Factor, Pitch};
use crate::to_anyhow;
use anyhow::anyhow;
use num_rational::Ratio;
use serde::Serialize;
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use winnow::combinator::{alt, delimited, eof, fail, opt, peek, preceded, separated, terminated};
use winnow::token::one_of;
use winnow::{Parser, combinator};

// Pass2 Step 1: Pass 2 of parsing uses the output of pass 1 as its input. The winnow crate allows
// any slice to be used as an input, but a lot of features aren't available if you use something
// other than `&str` or `&[u8]`. In particular, with rust 1.89 and winnow 0.7, using `map` on
// parsers or returning parsers directly turns out to be very hard. Attempting to define traits like
// the parser and intermediate parser traits from pass 1 results in rust errors about limitations of
// the trait solver that will be resolved in a future release. Any function that takes a mutable
// reference to the input type and returns a `winnow::Result` is a parser, and most of the basic
// combinators work with those. Search for Pass2 Step 2.
type Input2<'a, 's> = &'a [Token1<'s>];
pub type Token2<'s> = Spanned<Token<'s, Pass2<'s>>>;

#[derive(Serialize, Debug, Clone)]
pub enum Pass2<'s> {
    // Space, comments
    Space,
    Newline,
    Comment,
    Directive(RawDirective<'s>),
    NoteLine(NoteLine<'s>),
    DynamicLine(DynamicLine<'s>),
}
impl<'s> Display for Pass2<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Pass2::Directive(x) => write!(f, "Directive{{{x}}}"),
            Pass2::NoteLine(x) => write!(f, "NoteLine{{{x}}}"),
            Pass2::DynamicLine(x) => write!(f, "DynamicLine:{{{x}}}"),
            Pass2::Space | Pass2::Newline | Pass2::Comment => write!(f, "{self:?}"),
        }
    }
}

pub enum Degraded {
    Directive,
    Dynamic,
    Note,
    Definition,
    Misc,
}

/// Consumes one or more whitespace tokens consisting of spaces, comments, and possibly newlines.
fn space(allow_newline: bool) -> impl FnMut(&mut Input2) -> winnow::Result<()> {
    move |input| {
        let orig_len = input.len();
        while !input.is_empty() {
            match input[0].value.t {
                Pass1::Space | Pass1::Comment => consume_one(input),
                Pass1::Newline if allow_newline => consume_one(input),
                _ => break,
            }
        }
        if input.len() == orig_len {
            // No tokens
            return fail(input);
        }
        Ok(())
    }
}

fn space_only(input: &mut Input2) -> winnow::Result<()> {
    one_of(|x: Token1| matches!(x.value.t, Pass1::Space))
        .parse_next(input)
        .map(|_| ())
}

fn newline(input: &mut Input2) -> winnow::Result<()> {
    one_of(|x: Token1| matches!(x.value.t, Pass1::Newline))
        .parse_next(input)
        .map(|_| ())
}

fn optional_space(input: &mut Input2) -> winnow::Result<()> {
    opt(one_of(|x: Token1| matches!(x.value.t, Pass1::Space)))
        .parse_next(input)
        .map(|_| ())
}

fn some_space(input: &mut Input2) -> winnow::Result<()> {
    space(true).parse_next(input).map(|_| ())
}

fn newline_or_eof(input: &mut Input2) -> winnow::Result<()> {
    preceded(optional_space, alt((eof.map(|_| ()), newline)))
        .parse_next(input)
        .map(|_| ())
}

fn character(ch: char) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<char>> {
    move |input| {
        one_of(|x: Token1| x.value.raw.len() == 1 && x.value.raw.starts_with(ch))
            .parse_next(input)
            .map(|x| Spanned::new(x.span, ch))
    }
}

fn punctuation(input: &mut Input2) -> winnow::Result<Spanned<char>> {
    one_of(|x: Token1| matches!(x.value.t, Pass1::Punctuation))
        .parse_next(input)
        .map(|x| Spanned::new(x.span, x.value.raw.chars().next().unwrap()))
}

fn note_name<'s>() -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<&'s str>> {
    move |input| {
        one_of(|x: Token1| matches!(x.value.t, Pass1::NoteName))
            .parse_next(input)
            .map(|x| Spanned::new(x.span, x.value.raw))
    }
}

fn number(input: &mut Input2) -> winnow::Result<Spanned<u32>> {
    one_of(Pass1::is_number)
        .parse_next(input)
        .map(|tok| Spanned::new(tok.span, Pass1::get_number(&tok).unwrap()))
}

fn ratio_inner(
    allow_zero: bool,
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<Ratio<u32>>> {
    // Accept this as a ratio and consume the tokens as long as it is syntactically valid. If
    // there are problems report the errors.
    move |input| {
        (
            number,
            opt(preceded(character('.'), number)),
            opt(preceded(character('/'), number)),
        )
            .with_taken()
            .parse_next(input)
            .map(|((num_dec_t, num_frac_t, den_t), tokens)| {
                // We already know the numbers can be parsed into u32 from the first lexing pass.
                let span = tokens.get_span().unwrap();
                let num_dec: u32 = num_dec_t.value;
                let (num_frac, scale) = match num_frac_t {
                    None => (0, 1),
                    Some(frac) => {
                        let len = frac.span.end - frac.span.start;
                        if len > 3 {
                            diags.err(
                                code::NUM_FORMAT,
                                frac.span,
                                "a maximum of three decimal places is allowed",
                            );
                            // return any non-zero value to avoid a spurious zero error
                            (1, 10)
                        } else {
                            let v: u32 = frac.value;
                            (v, 10u32.pow(len as u32))
                        }
                    }
                };
                let mut numerator = match num_dec
                    .checked_mul(scale)
                    .and_then(|x| x.checked_add(num_frac))
                {
                    Some(x) => x,
                    None => {
                        diags.err(
                            code::NUM_FORMAT,
                            num_dec_t.span,
                            "too much precision for numerator",
                        );
                        1
                    }
                };
                if (!allow_zero || den_t.is_some()) && numerator == 0 {
                    diags.err(
                        code::NUM_FORMAT,
                        num_dec_t.span,
                        "zero not allowed as numerator",
                    );
                    numerator = 1;
                }
                let denominator: u32 = if let Some(den_t) = den_t {
                    let den: u32 = den_t.value;
                    if den == 0 {
                        diags.err(
                            code::NUM_FORMAT,
                            den_t.span,
                            "zero not allowed as denominator",
                        );
                        1
                    } else {
                        match den.checked_mul(scale) {
                            Some(x) => x,
                            None => {
                                diags.err(
                                    code::NUM_FORMAT,
                                    den_t.span,
                                    "too much precision for denominator",
                                );
                                1
                            }
                        }
                    }
                } else {
                    scale
                };

                Spanned::new(span, Ratio::new(numerator, denominator))
            })
    }
}

fn ratio(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<Ratio<u32>>> {
    ratio_inner(false, diags)
}

fn ratio_or_zero(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<Ratio<u32>>> {
    ratio_inner(true, diags)
}

fn exponent(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<Factor> {
    // Pass 2 Step 7: accept this as an exponent and consume the tokens as long as it is
    // syntactically valid. If there are problems report the errors. This is an example of the
    // pattern used frequently in pass 1 where we liberally match tokens and then perform
    // validation. Both this and the `ratio` parser have extensive validation. This allows us to
    // give precise, targeted error messages without breaking the flow of the parser. After studying
    // this function, resume with Pass 2 Step 8.
    move |input| {
        (
            opt((number, opt(preceded(character('/'), number)))),
            preceded(
                character('^'),
                (
                    opt(character('-')),
                    number,
                    preceded(character('|'), number),
                ),
            ),
        )
            .parse_next(input)
            .map(|(base, exp)| {
                // All parses can be safely unwrapped. We have verified that everything fits
                // in an i32.
                let (sign_t, exp_num_t, exp_den_t) = exp;
                let (base_num, base_den) = match base {
                    None => (2, 1),
                    Some((num, den)) => (
                        num.value,
                        match den {
                            None => 1,
                            Some(den) => den.value,
                        },
                    ),
                };
                let mut exp_num: i32 = exp_num_t.value as i32;
                let mut exp_den = exp_den_t.value as i32;
                if exp_den == 0 {
                    diags.err(
                        code::PITCH_SYNTAX,
                        exp_den_t.span,
                        "zero not allowed as exponent denominator",
                    );
                    exp_den = 1;
                }
                if sign_t.is_some() {
                    exp_num = -exp_num;
                };
                // We have checked all errors that are checked by Factor::new, so this should always
                // succeed.
                Factor::new(base_num, base_den, exp_num, exp_den)
                    .expect("uncaught pitch syntax error")
            })
    }
}

fn factor(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<Factor> {
    move |input| {
        alt((exponent(diags), ratio(diags).map(|x| Factor::from(x.value)))).parse_next(input)
    }
}

fn pitch_or_number(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<PitchOrNumber>> {
    // Pass 2 Step 6: this is a case where parser combinators make relatively easy what would
    // require a lot of work with a traditional grammar. All ratios parse as both ratios and
    // pitches. At this stage of processing, we don't know whether a pitch or a ratio will be
    // wanted, and you can't tell from a Pitch object whether it was originally specified as a
    // ratio. For example, the pitch `*4/3^0|19` is the ratio `4/3`, but it was written as a pitch.
    // If we ever directly called the `ratio` parser in an `alt` with `pitch`, if we called `ratio`
    // first, it would potentially consume part of a pitch, giving us unwanted results, and if we
    // called it second, it would never match because `pitch` would always match. Instead, we use
    // `peek` to see if this would also have matched as a ratio...
    move |input| {
        let as_ratio = peek(ratio(diags)).parse_next(input);
        preceded(
            opt(character('*')),
            separated(1.., factor(diags), character('*')),
        )
        .with_taken()
        .parse_next(input)
        .map(|(factors, tokens)| {
            // ...and then, if it matches as a pitch, we store it based on whether it also matched
            // as a ratio. The resulting `PitchOrNumber` can always be converted to a Pitch, and it
            // can also be converted to a `Ratio`, but only if it appeared literally as a ratio in
            // the input. This is not knowable by looking at the `Pitch` object. We have to store
            // the information at the time of parsing. After this logic was initially created, it
            // was further refined to allow recognition of straight integers for the same reason:
            // this can only be known at parse time. Continue to Pass 2 Step 7.
            let span = tokens.get_span().unwrap();
            let p = Pitch::new(factors);
            let r = if let Ok(r) = as_ratio {
                if tokens.len() == 1 {
                    // This must be a straight integer.
                    PitchOrNumber::Integer((*r.value.numer(), p))
                } else if r.span == span {
                    // This pitch is parseable as a ratio. Treat it as a ratio, and allow the
                    // semantic layer to upgrade it to a pitch later if needed.
                    PitchOrNumber::Ratio((r.value, p))
                } else {
                    PitchOrNumber::Pitch(p)
                }
            } else {
                PitchOrNumber::Pitch(p)
            };
            Spanned::new(span, r)
        })
    }
}

fn note_or_identifier<'s>(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<NoteOrIdentifier<'s>>> {
    move |input| {
        let as_identifier = peek(identifier).parse_next(input);
        note_octave(diags)
            .with_taken()
            .parse_next(input)
            .map(|(note, tokens)| {
                let span = tokens.get_span().unwrap();
                let r = match as_identifier {
                    Ok(i) if i.span == note.span => {
                        NoteOrIdentifier::Identifier(i.value, note.value)
                    }
                    _ => NoteOrIdentifier::Note(note.value),
                };
                Spanned::new(span, r)
            })
    }
}

fn identifier<'s>(input: &mut Input2<'_, 's>) -> winnow::Result<Spanned<Identifier<'s>>> {
    let t: winnow::Result<Spanned<Identifier<'s>>> =
        one_of(|x: Token1| matches!(x.value.t, Pass1::Identifier | Pass1::NoteName))
            .parse_next(input)
            .map(|t| {
                // This can be an identifier if the note name contains only characters that are allowed
                // in identifiers.
                Spanned::new(
                    t.span,
                    Identifier {
                        name: Cow::Borrowed(t.value.raw),
                    },
                )
            });
    if let Ok(v) = &t
        && !v
            .value
            .name
            .chars()
            .all(|c| c == '_' || c.is_ascii_alphanumeric())
    {
        return fail(input);
    }
    t
}

fn definition_start<'s>(input: &mut Input2<'_, 's>) -> winnow::Result<Spanned<String>> {
    one_of(|x: Token1| matches!(x.value.t, Pass1::DefinitionStart))
        .parse_next(input)
        .map(|t| Spanned::new(t.span, t.value.raw))
}

fn definition_end<'s>(input: &mut Input2<'_, 's>) -> winnow::Result<Spanned<String>> {
    one_of(|x: Token1| matches!(x.value.t, Pass1::DefinitionEnd))
        .parse_next(input)
        .map(|t| Spanned::new(t.span, t.value.raw))
}

fn string<'s>(
    _diags: &Diagnostics,
) -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<Cow<'s, str>>> {
    move |input| {
        one_of(Pass1::is_string)
            .parse_next(input)
            .map(|tok| Pass1::get_string(&tok).unwrap())
    }
}

fn zero<'s>() -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<u32>> {
    move |input| {
        let r = one_of(|x: Token1| matches!(x.value.t, Pass1::Number { n } if n.value == 0))
            .parse_next(input)
            .map(|tok| Spanned::new(tok.span, Pass1::get_number(&tok).unwrap()));
        if r.is_ok() && peek(character('.')).parse_next(input).is_ok() {
            return fail(input);
        }
        r
    }
}

fn param_value<'s>(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<ParamValue<'s>>> {
    move |input| {
        alt((
            string(diags).map(|x| ParamValue::String(x.value)),
            zero().map(|_| ParamValue::Zero),
            pitch_or_number(diags).map(|x| ParamValue::PitchOrNumber(x.value)),
            note_or_identifier(diags).map(|x| ParamValue::NoteOrIdentifier(x.value)),
        ))
        .with_taken()
        .parse_next(input)
        .map(|(value, tokens)| Spanned::new(tokens.get_span().unwrap(), value))
    }
}

fn param<'s>(diags: &Diagnostics) -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Param<'s>> {
    move |input| {
        (
            terminated(
                identifier,
                delimited(optional_space, character('='), optional_space),
            ),
            param_value(diags),
        )
            .parse_next(input)
            .map(|(key, value)| Param { key, value })
    }
}

fn directive<'s>(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<RawDirective<'s>>> {
    // Pass2 Step 5: this is an example of how our parsers look in Pass 2. Because we can't directly
    // return opaque types that implement some kind of Parser2, as we did with Parser1 in pass 1 (we
    // could maybe do it, but it would be lots of extra trait implementations, and as of initial
    // writing, there are rust-level trait solver issues as well), we make parsers by implementing
    // functions that take an input slice and return a result. This means we are responsible for
    // calling `parse_next` ourselves, and we don't have the convenience of abstracting away
    // `with_taken` when we need it. That creates a small amount of additional boilerplate. Most of
    // our parsers will return closures like this one. Inside the closures, we can still use normal
    // winnow combinators. See the winnow docs to understand what these functions do. They are
    // mostly self-explanatory. You can follow into these various parsers. Start by stepping into
    // `param` and follow the path. Then continue with Pass2 Step 6.
    move |input| {
        (
            identifier,
            delimited(
                delimited(optional_space, character('('), opt(some_space)),
                separated(0.., param(diags), some_space),
                (opt(some_space), character(')')),
            ),
        )
            .with_taken()
            .parse_next(input)
            .map(|items: ((_, Vec<_>), _)| {
                let ((name, params), tokens) = items;
                let span = tokens.get_span().unwrap();
                Spanned::new(
                    span,
                    RawDirective {
                        name,
                        params,
                        block: None,
                    },
                )
            })
    }
}

fn octave(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<i8>> {
    move |input| {
        (alt((character('\''), character(','), fail)), opt(number))
            .parse_next(input)
            .map(|(sym, num)| {
                let mut span = sym.span;
                let mut count: i8 = if let Some(n) = num {
                    span.end = n.span.end;
                    let count: i8 = if let Ok(n) = i8::try_from(n.value) {
                        n
                    } else {
                        diags.err(code::NOTE_SYNTAX, n.span, "octave count is too large");
                        1
                    };
                    if count == 0 {
                        // It can be zero, but not explicitly zero.
                        diags.err(code::NOTE_SYNTAX, n.span, "octave count may not be zero");
                    }
                    count
                } else {
                    1
                };
                if sym.value == ',' {
                    count = -count;
                }

                Spanned::new(span, count)
            })
    }
}

fn note_modifiers(
    modifiers: Vec<Spanned<char>>,
    diags: &Diagnostics,
) -> Vec<Spanned<NoteModifier>> {
    let mut result = Vec::new();
    for ch in modifiers {
        let span: Span = ch.span;
        let modifier = match ch.value {
            '>' => NoteModifier::Accent,
            '&' => NoteModifier::Glide,
            '^' => NoteModifier::Marcato,
            '.' => NoteModifier::Shorten,
            '~' => NoteModifier::Tie,
            _ => {
                diags.err(
                    code::NOTE_SYNTAX,
                    span,
                    format!("invalid note modifier '{}'", ch.value),
                );
                continue;
            }
        };
        result.push(Spanned::new(span, modifier));
    }
    result
}

fn hold<'s>(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<Note<'s>>> {
    |input| {
        (
            opt(terminated(ratio(diags), character(':'))),
            character('~'),
        )
            .parse_next(input)
            .map(|(duration, ch)| {
                let span = model::merge_spans(&[duration.get_span(), ch.get_span()]).unwrap();
                Spanned::new(span, Note::Hold(Hold { duration, ch }))
            })
    }
}

fn bar_check() -> impl FnMut(&mut Input2) -> winnow::Result<Span> {
    |input| character('|').parse_next(input).map(|c| c.span)
}

fn note_octave<'s>(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<NoteOctave<'s>>> {
    |input| {
        (note_name(), opt(octave(diags)))
            .parse_next(input)
            .map(|(name, octave)| {
                let span = model::merge_spans(&[name.get_span(), octave.get_span()]).unwrap();
                let name = Spanned::new(name.span, Cow::Borrowed(name.value));
                Spanned::new(span, NoteOctave { name, octave })
            })
    }
}

fn regular_note<'s>(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<Note<'s>>> {
    |input| {
        (
            opt(terminated(ratio(diags), character(':'))),
            note_octave(diags),
            opt(preceded(
                character(':'),
                combinator::repeat(1.., punctuation),
            )),
        )
            .parse_next(input)
            .map(|items| {
                let (duration, note, modifiers) = items;
                // This merges these spans. Since `name` is definite set, we can safely unwrap
                // the result.
                let modifier_span = modifiers.get_span();
                let span =
                    model::merge_spans(&[duration.get_span(), note.get_span(), modifier_span])
                        .unwrap();
                let modifiers = modifiers
                    .map(|x| note_modifiers(x, diags))
                    .unwrap_or_default();
                Spanned::new(
                    span,
                    Note::Regular(RegularNote {
                        duration,
                        note,
                        modifiers,
                    }),
                )
            })
    }
}

fn note<'s>(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<Note<'s>>> {
    move |input| {
        alt((
            regular_note(diags),
            hold(diags),
            bar_check().map(|span| Spanned::new(span, Note::BarCheck(span))),
            fail,
        ))
        .parse_next(input)
    }
}

fn note_leader<'s>() -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<NoteLeader<'s>>> {
    // Pass 2 Step 9: inner spans. To keep Pass1 tokens copiable and to keep them from containing
    // multiple slices to the same input source (which would require the `Pass1` type to have a
    // namespace parameter), some Pass1 tokens contain inner spans. The inner spans are always
    // ranges relative to the original input as they should be directly usable for error messages.
    // If we want to grab the slice of the original source when all we have is the bytes for that
    // token, we need to adjust the span. To be more concrete, if a string token, for example, has a
    // span of 10..20, the inside of the string not including the quotation marks will have the span
    // 11..19. That could be applied to the original input, but we don't have that inside the string
    // token. We only have what was matched by the string token. To help with that, our Span type
    // has a `relative_to` method that returns a span relative to an outer span. You can see its use
    // here to get the text corresponding to the name span and the note span. This allows us to
    // construct a NoteLeader from the Pass1 token that contains spanned elements, which makes it
    // possible for the semantic layer to use those spans for specific downstream error messages.
    // This is exercised nontrivially in the test suite. That concludes the tour of pass 2. It
    // should now be possible to understand the remaining parsers. Look at the tests for the various
    // passes, the overall parser tests, and comments for additional passes.
    move |input| {
        one_of(Pass1::is_note_leader).parse_next(input).map(|tok| {
            let (name_span, note) = Pass1::get_note_leader(&tok).unwrap();
            Spanned::new(
                tok.span,
                NoteLeader {
                    name: Spanned::new(name_span, &tok.value.raw[name_span.relative_to(tok.span)]),
                    note,
                },
            )
        })
    }
}

fn require_spaces<T: Debug + Serialize>(
    diags: &Diagnostics,
    v: Vec<(Option<()>, Spanned<T>)>,
) -> Vec<Spanned<T>> {
    // Space is required, but don't want omission to prevent the line from being
    // recognized, causing spurious errors or preventing other parsing.
    v.into_iter()
        .map(|(spc, item)| {
            if spc.is_none() {
                diags.err(
                    code::SCORE_SYNTAX,
                    item.span,
                    "this item must be preceded by a space",
                );
            }
            item
        })
        .collect()
}

fn note_line<'s>(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<NoteLine<'s>>> {
    move |input| {
        (
            note_leader(),
            terminated(
                combinator::repeat(1.., (opt(space_only), note(diags))),
                terminated(opt(space(false)), newline_or_eof),
            ),
        )
            .with_taken()
            .parse_next(input)
            .map(|((leader, notes), tokens): ((_, Vec<_>), Input2)| {
                let span = tokens.get_span().unwrap();
                // Space is required, but don't want omission to prevent the line from being
                // recognized, causing spurious errors or preventing other parsing.
                let notes = require_spaces(diags, notes);
                Spanned::new(span, NoteLine { leader, notes })
            })
    }
}

fn regular_dynamic(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<Dynamic>> {
    |input| {
        (
            one_of(|x: Token1| matches!(x.value.t, Pass1::Number { .. })),
            character('@'),
            ratio_or_zero(diags),
            opt(alt((character('>'), character('<')))),
        )
            .parse_next(input)
            .map(|items| {
                let (level_t, _, position, change_t) = items;
                let level: u8 = match Pass1::get_number(&level_t).unwrap() {
                    x if x > 127 => {
                        diags.err(
                            code::DYNAMIC_SYNTAX,
                            level_t.span,
                            "dynamic value must be <= 127",
                        );
                        127
                    }
                    x => x as u8,
                };
                let change = change_t.map(|t| {
                    Spanned::new(
                        t.span,
                        if t.value == '<' {
                            DynamicChange::Crescendo
                        } else {
                            DynamicChange::Diminuendo
                        },
                    )
                });
                // This merges these spans. We can safely unwrap since some values are definitely
                // set.
                let span = model::merge_spans(&[
                    level_t.get_span(),
                    position.get_span(),
                    change_t.get_span(),
                ])
                .unwrap();
                Spanned::new(
                    span,
                    Dynamic::Regular(RegularDynamic {
                        level: Spanned::new(level_t.span, level),
                        change,
                        position,
                    }),
                )
            })
    }
}

fn dynamic(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<Dynamic>> {
    move |input| {
        alt((
            regular_dynamic(diags),
            bar_check().map(|span| Spanned::new(span, Dynamic::BarCheck(span))),
            fail,
        ))
        .parse_next(input)
    }
}

fn dynamic_leader<'s>()
-> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<DynamicLeader<'s>>> {
    move |input| {
        one_of(Pass1::is_dynamic_leader)
            .parse_next(input)
            .map(|tok| {
                let name_span = Pass1::get_dynamic_leader(&tok).unwrap();
                Spanned::new(
                    tok.span,
                    DynamicLeader {
                        name: Spanned::new(
                            name_span,
                            &tok.value.raw[name_span.relative_to(tok.span)],
                        ),
                    },
                )
            })
    }
}

fn dynamic_line<'s>(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<DynamicLine<'s>>> {
    move |input| {
        (
            dynamic_leader(),
            terminated(
                combinator::repeat(1.., (opt(space_only), dynamic(diags))),
                terminated(opt(space(false)), newline_or_eof),
            ),
        )
            .with_taken()
            .parse_next(input)
            .map(|((leader, dynamics), tokens): ((_, Vec<_>), Input2)| {
                let span = tokens.get_span().unwrap();
                let dynamics = require_spaces(diags, dynamics);
                Spanned::new(span, DynamicLine { leader, dynamics })
            })
    }
}

fn scale_note<'s>(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<ScaleNote<'s>>> {
    |input| {
        let r1: Spanned<ScaleNote> = terminated(
            (
                preceded(optional_space, pitch_or_number(diags)),
                terminated(
                    combinator::repeat(1.., preceded(space_only, note_name())),
                    opt(some_space),
                ),
            ),
            optional_space,
        )
        .parse_next(input)
        .map(|items: (_, Vec<_>)| {
            let (pitch, note_names) = items;
            let span =
                model::merge_spans(&[pitch.get_span(), note_names.as_slice().get_span()]).unwrap();
            Spanned::new(span, ScaleNote { pitch, note_names })
        })?;
        Ok(r1)
    }
}

fn check_scale_block<'s>(diags: &Diagnostics, input: &mut Input2<'_, 's>) -> bool {
    peek((
        opt(some_space),
        definition_start,
        opt(some_space),
        pitch_or_number(diags),
    ))
    .parse_next(input)
    .is_ok()
}

fn scale_block<'s>(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<ScaleBlock<'s>>> {
    |input| {
        (
            delimited(opt(some_space), definition_start, opt(some_space)),
            combinator::repeat(1.., scale_note(diags)),
            preceded(optional_space, definition_end),
        )
            .parse_next(input)
            .map(|items: (_, Vec<_>, _)| {
                let (start, notes, end) = items;
                let span = Span::from(start.span.start..end.span.end);
                Spanned::new(
                    span,
                    ScaleBlock {
                        notes: Spanned::new(span, notes),
                    },
                )
            })
    }
}

fn layout_item<'s>(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<LayoutItem<'s>>> {
    |input| {
        (
            opt(character('@')),
            alt((
                character('~')
                    .map(|x| Spanned::<LayoutItemType>::new(x.span, LayoutItemType::Empty(x.span))),
                note_octave(diags)
                    .map(|x| Spanned::<LayoutItemType>::new(x.span, LayoutItemType::Note(x))),
            )),
        )
            .parse_next(input)
            .map(|(anchor, item)| {
                let span = model::merge_spans(&[anchor.get_span(), item.get_span()]).unwrap();
                Spanned::new(
                    span,
                    LayoutItem {
                        item: item.value,
                        is_anchor: anchor.map(|x| x.span),
                    },
                )
            })
    }
}

fn check_layout_block<'s>(diags: &Diagnostics, input: &mut Input2<'_, 's>) -> bool {
    peek((
        opt(some_space),
        definition_start,
        opt(some_space),
        layout_item(diags),
    ))
    .parse_next(input)
    .is_ok()
}

fn layout_line<'s>(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<Vec<Spanned<LayoutItem<'s>>>>> {
    |input| {
        terminated(
            combinator::repeat(1.., preceded(optional_space, layout_item(diags))),
            terminated(opt(space(false)), newline_or_eof),
        )
        .parse_next(input)
        .map(|items: Vec<Spanned<LayoutItem>>| {
            let span = items.as_slice().get_span().unwrap();
            Spanned::new(span, items)
        })
    }
}

fn layout_block<'s>(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2<'_, 's>) -> winnow::Result<Spanned<LayoutBlock<'s>>> {
    |input| {
        (
            delimited(opt(some_space), definition_start, (optional_space, newline)),
            combinator::repeat(1.., layout_line(diags)),
            preceded(optional_space, definition_end),
        )
            .parse_next(input)
            .map(|items: (_, Vec<_>, _)| {
                let (start, rows, end) = items;
                let span = Span::from(start.span.start..end.span.end);
                Spanned::new(
                    span,
                    LayoutBlock {
                        rows: Spanned::new(span, rows),
                    },
                )
            })
    }
}

fn degraded_top_level(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<()> {
    // These are things that can appear at the top level with higher-level things before lower-level
    // things. This is useful for scanning through tokens in degraded mode.
    move |input| {
        alt((
            space(false),
            param(diags).map(|_| ()),
            character('=').map(|_| ()),
            identifier.map(|_| ()),
            string(diags).map(|_| ()),
            pitch_or_number(diags).map(|_| ()),
        ))
        .parse_next(input)
    }
}

fn degraded_directive(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<()> {
    // Pass 2 Step 8: degraded mode. Match on things that may appear in a directive, stopping when
    // we hit a closed parenthesis, and reporting errors for surprises. There are several sharp
    // edges to be aware of:
    // - all branches of `alt` have to return the same type. In this case, we don't care about any
    //   of the values. You can see in the `degraded_top_level` function that we map the output of
    //   each branch of `alt` to the unit type, but you can find other calls to `map` that are less
    //   trivial, such as the param_value or factor parsers.
    // - winnow panics of the `repeat` combinator doesn't consume any tokens. That means that, if
    //   you pass `alt` to `repeat`, you must ensure that none of the branches of `alt` contain
    //   `opt`, since `opt` can succeed and match zero tokens. Specifically, calling
    //   `repeat(alt(opt(...)))` will cause `opt` to match if nothing else matches, which will cause
    //   a panic.
    //
    // Notice what's happening here. The main repeat loop keeps matching tokens up to a `")"`. When
    // it encounters one of the valid top-level tokens (in `degraded_top_level`), it just discards
    // it, but this allows those matchers to report errors they may notice. For example, if there's
    // an invalid pitch or a broken string literal, we'll still see those errors. If any other token
    // appears, which presumably happens or else the original `directive` parser would have matched,
    // we report a syntax error and consume the token. This produces better error recovery. It is
    // nontrivially tested in the test suite. All the degraded mode parsers work in this way.
    //
    // Continue with Pass 2 Step 9 for additional notes.
    move |input| {
        terminated(
            combinator::repeat(
                1..,
                alt((
                    newline_or_eof,
                    degraded_top_level(diags),
                    one_of(|x: Token1| x.value.raw != ")").map(|tok: Token1| {
                        diags.err(code::SYNTAX, tok.span, diagnostics::SYNTAX_ERROR);
                    }),
                )),
            )
            .map(|_: Vec<_>| ()),
            character(')'),
        )
        .parse_next(input)
        .map(|_| ())
    }
}

fn degraded_note(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<()> {
    move |input| {
        terminated(
            combinator::repeat(
                1..,
                alt((
                    space(false),
                    note(diags).map(|_| ()),
                    one_of(|x: Token1| !matches!(x.value.t, Pass1::Newline)).map(|tok: Token1| {
                        diags.err(code::SCORE_SYNTAX, tok.span, "unexpected item in note line");
                    }),
                )),
            )
            .map(|_: Vec<_>| ()),
            newline_or_eof,
        )
        .parse_next(input)
        .map(|_| ())
    }
}

fn degraded_dynamic(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<()> {
    move |input| {
        terminated(
            combinator::repeat(
                1..,
                alt((
                    space(false),
                    dynamic(diags).map(|_| ()),
                    one_of(|x: Token1| !matches!(x.value.t, Pass1::Newline)).map(|tok: Token1| {
                        diags.err(
                            code::SCORE_SYNTAX,
                            tok.span,
                            "unexpected item in dynamic line",
                        );
                    }),
                )),
            )
            .map(|_: Vec<_>| ()),
            newline_or_eof,
        )
        .parse_next(input)
        .map(|_| ())
    }
}

fn degraded_definition(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<()> {
    move |input| {
        delimited(
            definition_start,
            combinator::repeat(
                1..,
                alt((
                    space(false),
                    newline.map(|_| ()),
                    pitch_or_number(diags).map(|_| ()),
                    character('|').map(|_| ()),
                    character('@').map(|_| ()),
                    character('~').map(|_| ()),
                    note_octave(diags).map(|_| ()),
                    one_of(|x: Token1| !matches!(x.value.t, Pass1::DefinitionEnd)).map(
                        |tok: Token1| {
                            diags.err(
                                code::SCORE_SYNTAX,
                                tok.span,
                                "unexpected item in definition block",
                            );
                        },
                    ),
                )),
            )
            .map(|_: Vec<_>| ()),
            newline_or_eof,
        )
        .parse_next(input)
        .map(|_| ())
    }
}

fn degraded_misc(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<()> {
    move |input| {
        terminated(
            combinator::repeat(
                1..,
                alt((
                    degraded_top_level(diags),
                    character('(').map(|_| ()),
                    character(')').map(|_| ()),
                    one_of(|x: Token1| !matches!(x.value.t, Pass1::Newline)).map(|tok: Token1| {
                        diags.err(code::SYNTAX, tok.span, diagnostics::SYNTAX_ERROR);
                    }),
                )),
            )
            .map(|_: Vec<_>| ()),
            character(')'),
        )
        .parse_next(input)
        .map(|_| ())
    }
}

fn consume_one<T>(items: &mut &[T]) {
    if !items.is_empty() {
        *items = &items[1..]
    }
}

fn promote<'s>(lt: &Token1<'s>, t: Pass2<'s>) -> Token2<'s> {
    Token::new_spanned(lt.value.raw, lt.span, t)
}

fn promote_and_consume_first<'s>(input: &mut Input2<'_, 's>, t: Pass2<'s>) -> Token2<'s> {
    let tok = promote(&input[0], t);
    consume_one(input);
    tok
}

/// Handle the current token, advancing input and appending to out as needed.
fn handle_token<'s>(
    src: &'s str,
    input: &mut Input2<'_, 's>,
    diags: &Diagnostics,
) -> Result<Token2<'s>, Degraded> {
    // Pass2 Step 3: this is called from the pass-2 main loop. It is similar to the pass-1 main loop
    // in that peeks at the first token to decide which branch to take. Here, we actually call
    // `peek` to look ahead a little farther to decide which branch to commit to.
    let tok = &input[0];
    match &tok.value.t {
        // Space, comments, and newlines are the same for all modes. In pass 1, we recognized these
        // by looking at a character. In pass 2, we recognize them by noticing the pass-1 tokens. We
        // could have used `any` to grab the first token, but there's no reason that we can't
        // simplify things and just advance input ourselves. In pass 1, we relied more heavily on
        // winnow for things like spans. In pass 2, we are getting spans by merging spans from
        // pass-1 tokens. You can see that promote_and_consume_first is manually copying payload and
        // advancing the input slice. The pass-1 parser returns tokens suitable to the lexer state,
        // so we don't have to repeat that logic here. We know what kind of state we're in based on
        // the next token.
        Pass1::Space => Ok(promote_and_consume_first(input, Pass2::Space)),
        Pass1::Newline => Ok(promote_and_consume_first(input, Pass2::Newline)),
        Pass1::Comment => Ok(promote_and_consume_first(input, Pass2::Comment)),
        Pass1::NoteName => {
            // Pass 1 only gives us the `NoteName` token at the top-level. The top level state
            // consists entirely of directives (after white space and comments have been handled).
            // We get better error messages if we know we're in something that looks like a
            // directive instead of a stray identifier, so look for the open parenthesis. This
            // enables us to fail into one of two different degraded modes.
            if peek((identifier, optional_space, character('(')))
                .parse_next(input)
                .is_ok()
            {
                // Try to parse as a directive.
                if let Ok(mut d) = directive(diags).parse_next(input) {
                    // We got a directive. If the next thing is a definition, attach it.
                    let t: Option<Spanned<DataBlock>> = if check_scale_block(diags, input) {
                        scale_block(diags)
                            .parse_next(input)
                            .ok()
                            .map(|x| Spanned::new(x.span, DataBlock::Scale(x.value)))
                    } else if check_layout_block(diags, input) {
                        layout_block(diags)
                            .parse_next(input)
                            .ok()
                            .map(|x| Spanned::new(x.span, DataBlock::Layout(x.value)))
                    } else {
                        None
                    };
                    if let Some(data_block) = t {
                        // Extend the span to cover the data block, and attach it.
                        d.span.end = data_block.span.end;
                        d.value.block = Some(data_block);
                    }
                    Ok(Token::new_spanned(
                        &src[d.span],
                        d.span,
                        Pass2::Directive(d.value),
                    ))
                } else {
                    // We didn't get a directive, but we know we're in something that looks like a
                    // directive. Fall into a directive-specific degraded mode.
                    diags.err(
                        code::TOPLEVEL_SYNTAX,
                        tok.span,
                        "unable to parse as directive; expected directive(k=v ...)",
                    );
                    Err(Degraded::Directive)
                }
            } else {
                // If we didn't see an open parenthesis, fall back to a different degraded mode. The
                // differences are explained in later comments.
                diags.err(code::TOPLEVEL_SYNTAX, tok.span, "expected a directive");
                Err(Degraded::Misc)
            }
        }
        Pass1::NoteLeader { .. } => {
            // This logic is the same as the directive logic but for note lines. The presence of a
            // note leader tells us we should be in a note line. We don't need a peek as above
            // because the single token is sufficient. That's because pass 1 already recognized the
            // entire leader token `[part.note]`. As for directive, we have a note-specific degraded
            // mode if the rule didn't match.
            if let Ok(x) = note_line(diags).parse_next(input) {
                Ok(Token::new_spanned(
                    &src[x.span],
                    x.span,
                    Pass2::NoteLine(x.value),
                ))
            } else {
                diags.err(code::SCORE_SYNTAX, tok.span, "unable to parse as note line");
                Err(Degraded::Note)
            }
        }
        Pass1::DynamicLeader { .. } => {
            // This is the same as the note leader logic except for dynamics.
            if let Ok(x) = dynamic_line(diags).parse_next(input) {
                Ok(Token::new_spanned(
                    &src[x.span],
                    x.span,
                    Pass2::DynamicLine(x.value),
                ))
            } else {
                diags.err(
                    code::SCORE_SYNTAX,
                    tok.span,
                    "unable to parse as dynamic line",
                );
                Err(Degraded::Dynamic)
            }
        }
        Pass1::DefinitionStart => {
            diags.err(
                code::DEFINITION_SYNTAX,
                tok.span,
                "incorrect or misplaced definition block",
            );
            Err(Degraded::Definition)
        }
        _ => {
            // If we get anything else, fall into degraded mode. Continue with Pass2 Step 4.
            diags.err(code::SYNTAX, tok.span, diagnostics::SYNTAX_ERROR);
            Err(Degraded::Misc)
        }
    }
}

/// Helper function for the Pitch struct
pub fn parse_pitch(s: &str) -> anyhow::Result<Pitch> {
    let mut p: Option<Pitch> = None;
    let mut diags: Option<Diagnostics> = None;
    match pass1::parse1(s) {
        Ok(tokens) => {
            let input = tokens.as_slice();
            let d = Diagnostics::new();
            let pr = pitch_or_number(&d).parse(input);
            match pr {
                Ok(pr) => {
                    if d.has_errors() {
                        diags = Some(d);
                    } else {
                        p = Some(pr.value.into_pitch());
                    }
                }
                Err(_) => diags = Some(d),
            };
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
        anyhow!("unable to parse as pitch")
    };
    Err(anyhow!("{s}: {err}"))
}

pub fn parse2<'s>(src: &'s str) -> Result<Vec<Token2<'s>>, Diagnostics> {
    // Pass2 Step 2: this function performs pass 2 of the parsing. For ergonomic and lifetime
    // handling reasons, it must take the original input source. We first do pass1 parsing, and if
    // that fails, we stop. Otherwise, we continue with a simple parser for pass 2. The job of the
    // pass-2 parser is to recognize higher-level token types that are composed of lexical tokens.
    // This is approximately like parsing in a traditional scanner/parsers/static semantics model,
    // but we are not generating a full abstract syntax tree and are still doing some lexical work.
    // This function goes from &str -> Vec<Token1> -> Vec<Token2>.

    let low_tokens = pass1::parse1(src)?;
    let diags = Diagnostics::new();
    let mut input = low_tokens.as_slice();
    let mut out: Vec<Token2> = Vec::new();

    // Proceed to Pass2 Step 3 in handle_token.
    while !input.is_empty() {
        match handle_token(src, &mut input, &diags) {
            Ok(tok) => {
                model::trace(format!("lex pass 2: {tok}"));
                out.push(tok);
            }
            Err(mode) => {
                // Pass2 Step 4: degraded mode. Call a more relaxed parser that accepts tokens that
                // don't belong to it. The Directive degraded mode tries to scan until it finds a
                // closed parenthesis. It is only invoked when we found the open parenthesis. The
                // other degraded mode parsers expect the kinds of tokens we would expect in that
                // type of line but return to normal parsing on newline or EOF. Search for Pass2
                // Step 5.
                let tok = match mode {
                    Degraded::Directive => degraded_directive(&diags).parse_next(&mut input),
                    Degraded::Dynamic => degraded_dynamic(&diags).parse_next(&mut input),
                    Degraded::Note => degraded_note(&diags).parse_next(&mut input),
                    Degraded::Definition => degraded_definition(&diags).parse_next(&mut input),
                    Degraded::Misc => degraded_misc(&diags).parse_next(&mut input),
                };
                if tok.is_err() {
                    // Discard the next token and continue.
                    consume_one(&mut input);
                }
            }
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
