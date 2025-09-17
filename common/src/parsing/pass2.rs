use crate::parsing::model::{
    Diagnostics, Directive, GetSpan, Hold, Note, NoteBehavior, NoteLeader, NoteLine, NoteOption,
    Param, ParamValue, PitchOrRatio, RegularNote, Span, Spanned, Token, code,
};
use crate::parsing::pass1::{Pass1, Token1};
use crate::parsing::{model, pass1};
use crate::pitch::{Factor, Pitch};
use crate::to_anyhow;
use anyhow::anyhow;
use num_rational::Ratio;
use std::fmt::Debug;
use winnow::combinator::{alt, delimited, fail, opt, peek, preceded, separated, terminated};
use winnow::token::{one_of, take_while};
use winnow::{Parser, combinator};

type Input2<'a, 's> = &'a [Token1<'s>];
pub type Token2<'s> = Spanned<Token<'s, Pass2>>;

#[derive(Debug, Clone)]
pub enum Pass2 {
    // Space, comments
    Space,
    Newline,
    Comment,
    Directive(Directive),
    NoteLine(NoteLine),
}

fn optional_space(input: &mut Input2) -> winnow::Result<()> {
    opt(one_of(|x: Token1| matches!(x.value.t, Pass1::Space)))
        .parse_next(input)
        .map(|_| ())
}

fn space(input: &mut Input2) -> winnow::Result<()> {
    one_of(|x: Token1| matches!(x.value.t, Pass1::Space))
        .parse_next(input)
        .map(|_| ())
}

fn optional_space_or_newline(input: &mut Input2) -> winnow::Result<()> {
    take_while(0.., |x: Token1| {
        matches!(x.value.t, Pass1::Space | Pass1::Newline | Pass1::Comment)
    })
    .parse_next(input)
    .map(|_| ())
}

fn param_separator(input: &mut Input2) -> winnow::Result<()> {
    (optional_space, character(','), optional_space_or_newline)
        .parse_next(input)
        .map(|_| ())
}

fn newline() -> impl FnMut(&mut Input2) -> winnow::Result<()> {
    move |input| {
        one_of(|x: Token1| matches!(x.value.t, Pass1::Newline))
            .parse_next(input)
            .map(|_| ())
    }
}

fn character(ch: char) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<char>> {
    move |input| {
        one_of(|x: Token1| x.value.raw.len() == 1 && x.value.raw.starts_with(ch))
            .parse_next(input)
            .map(|x| Spanned::new(x.span, ch))
    }
}

fn number() -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<u32>> {
    move |input| {
        one_of(Pass1::is_number)
            .parse_next(input)
            .map(|tok| Spanned::new(tok.span, Pass1::get_number(&tok).unwrap()))
    }
}

fn ratio(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<Ratio<u32>>> {
    // Accept this as a ratio and consume the tokens as long as it is syntactically valid. If
    // there are problems report the errors.
    move |input| {
        (
            number(),
            opt(preceded(character('.'), number())),
            opt(preceded(character('/'), number())),
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
                                code::NUMBER,
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
                let mut numerator =
                    match u32::try_from(num_dec as u64 * scale as u64 + num_frac as u64) {
                        Ok(x) => x,
                        Err(_) => {
                            diags.err(
                                code::NUMBER,
                                num_dec_t.span,
                                "insufficient precision for numerator",
                            );
                            1
                        }
                    };
                if numerator == 0 {
                    diags.err(
                        code::NUMBER,
                        num_dec_t.span,
                        "zero not allowed as numerator",
                    );
                    numerator = 1;
                }
                let denominator: u32 = if let Some(den_t) = den_t {
                    let den: u32 = den_t.value;
                    if den == 0 {
                        diags.err(code::NUMBER, den_t.span, "zero not allowed as denominator");
                        1
                    } else {
                        match u32::try_from(den as u64 * scale as u64) {
                            Ok(x) => x,
                            Err(_) => {
                                diags.err(
                                    code::NUMBER,
                                    den_t.span,
                                    "insufficient precision for denominator",
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

fn exponent(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<Factor> {
    // Accept this as an exponent and consume the tokens as long as it is syntactically valid. If
    // there are problems report the errors.
    move |input| {
        (
            opt((number(), opt(preceded(character('/'), number())))),
            preceded(
                character('^'),
                (
                    opt(character('-')),
                    number(),
                    preceded(character('|'), number()),
                ),
            ),
        )
            .parse_next(input)
            .map(|(base, exp)| {
                // All parses can be safely unwrapped. We have verified that everything fits
                // in an i32.
                let (sign_t, exp_num_t, exp_den_t) = exp;
                let mut span_start = exp_num_t.span.start;
                let span_end = exp_den_t.span.end;
                let (base_num, base_den) = match base {
                    None => (2, 1),
                    Some((num, den)) => {
                        span_start = num.span.start;
                        (
                            num.value,
                            match den {
                                None => 1,
                                Some(den) => den.value,
                            },
                        )
                    }
                };
                let mut exp_num: i32 = exp_num_t.value as i32;
                let exp_den = exp_den_t.value as i32;
                if let Some(c) = sign_t {
                    span_start = c.span.start;
                    exp_num = -exp_num;
                };
                match Factor::new(base_num, base_den, exp_num, exp_den) {
                    Ok(f) => f,
                    Err(e) => {
                        diags.err(code::PITCH, span_start..span_end, e.to_string());
                        Factor::new(1, 1, 1, 1).unwrap()
                    }
                }
            })
    }
}

fn factor(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<Factor> {
    move |input| {
        alt((exponent(diags), ratio(diags).map(|x| Factor::from(x.value)))).parse_next(input)
    }
}

fn pitch_or_ratio(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<PitchOrRatio> {
    move |input| {
        let as_ratio = peek(ratio(diags)).parse_next(input);
        preceded(
            opt(character('*')),
            separated(1.., factor(diags), character('*')),
        )
        .with_taken()
        .parse_next(input)
        .map(|(factors, tokens)| {
            let span = tokens.get_span().unwrap();
            let p = Pitch::new(factors);
            if let Ok(r) = as_ratio {
                if r.span == span {
                    // This pitch is parseable as a ratio. Treat it as a ration, and allow the
                    // semantic layer to upgrade it to a pitch later if needed.
                    PitchOrRatio::Ratio((r.value, p))
                } else {
                    PitchOrRatio::Pitch(p)
                }
            } else {
                PitchOrRatio::Pitch(p)
            }
        })
    }
}

fn identifier<'s>(input: &mut Input2<'_, 's>) -> winnow::Result<Spanned<String>> {
    one_of(|x: Token1| matches!(x.value.t, Pass1::Identifier))
        .parse_next(input)
        .map(|t| Spanned::new(t.span, t.value.raw))
}

fn string(_diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<String>> {
    move |input| {
        one_of(Pass1::is_string)
            .parse_next(input)
            .map(|tok| Pass1::get_string(&tok).unwrap())
    }
}

fn param_value(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<ParamValue>> {
    move |input| {
        alt((
            string(diags).map(|x| ParamValue::String(x.value)),
            pitch_or_ratio(diags).map(ParamValue::PitchOrRatio),
        ))
        .with_taken()
        .parse_next(input)
        .map(|(value, tokens)| Spanned::new(tokens.get_span().unwrap(), value))
    }
}

fn param(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<Param> {
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

fn directive(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<Directive>> {
    move |input| {
        (
            terminated(
                identifier,
                (optional_space, character('('), optional_space_or_newline),
            ),
            terminated(
                separated(0.., param(diags), param_separator),
                (
                    opt(param_separator),
                    optional_space_or_newline,
                    character(')'),
                ),
            ),
        )
            .with_taken()
            .parse_next(input)
            .map(|((name, params), tokens)| {
                let span = tokens.get_span().unwrap();
                Spanned::new(span, Directive { name, params })
            })
    }
}

fn octave(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<i8>> {
    move |input| {
        (alt((character('\''), character(','), fail)), opt(number()))
            .parse_next(input)
            .map(|(sym, num)| {
                let mut span = sym.span;
                let mut count: i8 = if let Some(n) = num {
                    span.end = n.span.end;
                    let count: i8 = if let Ok(n) = i8::try_from(n.value) {
                        n
                    } else {
                        diags.err(code::SYNTAX, n.span, "octave count is too large");
                        1
                    };
                    if count == 0 {
                        // It can be zero, but not explicitly zero.
                        diags.err(code::SYNTAX, n.span, "octave count may not be zero");
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

fn note_options(
    diags: &Diagnostics,
) -> impl FnMut(&mut Input2) -> winnow::Result<Vec<Spanned<NoteOption>>> {
    move |input| {
        one_of(Pass1::is_note_options).parse_next(input).map(|tok| {
            let inner_span = Pass1::get_note_options(&tok).unwrap();
            let data = &tok.value.raw[inner_span.relative_to(tok.span)];
            if data.is_empty() {
                diags.err(code::LEXICAL, tok.span, "note options may not be empty");
            }
            let mut result = Vec::new();
            let mut offset = inner_span.start;
            for ch in data.chars() {
                let span: Span = (offset..offset + ch.len_utf8()).into();
                match ch {
                    '>' => result.push(Spanned::new(span, NoteOption::Accent)),
                    '^' => result.push(Spanned::new(span, NoteOption::Marcato)),
                    _ => diags.err(code::SYNTAX, span, format!("invalid note option '{ch}'")),
                }
                offset = span.end;
            }
            result
        })
    }
}

fn note_behavior() -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<NoteBehavior>> {
    move |input| {
        alt((
            character('>').map(|c| Spanned::new(c.span, NoteBehavior::Slide)),
            character('~').map(|c| Spanned::new(c.span, NoteBehavior::Sustain)),
            fail,
        ))
        .parse_next(input)
    }
}

fn hold(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<Note>> {
    |input| {
        (
            opt(terminated(ratio(diags), character(':'))),
            character('~'),
        )
            .parse_next(input)
            .map(|(duration, h)| {
                let span = model::merge_spans(&[duration.get_span(), h.get_span()]).unwrap();
                Spanned::new(span, Note::Hold(Hold { duration }))
            })
    }
}

fn bar_check() -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<Note>> {
    |input| {
        character('|')
            .parse_next(input)
            .map(|c| Spanned::new(c.span, Note::BarCheck(c.span)))
    }
}

fn regular_note(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<Note>> {
    |input| {
        (
            opt(terminated(ratio(diags), character(':'))),
            one_of(|x: Token1| matches!(x.value.t, Pass1::NoteName)),
            opt(octave(diags)),
            opt(note_options(diags)),
            opt(note_behavior()),
        )
            .parse_next(input)
            .map(|items| {
                let (duration, name, octave, options, behavior) = items;
                let name = Spanned::new(name.span, name.value.raw);
                // This merges these spans. Since `name` is definite set, we can safely unwrap
                // the result.
                let span = model::merge_spans(&[
                    duration.get_span(),
                    name.get_span(),
                    octave.get_span(),
                    options.get_span(),
                    behavior.get_span(),
                ])
                .unwrap();
                Spanned::new(
                    span,
                    Note::Regular(RegularNote {
                        duration,
                        name,
                        octave,
                        options: options.unwrap_or_default(),
                        behavior,
                    }),
                )
            })
    }
}

fn note(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<Note>> {
    move |input| alt((regular_note(diags), hold(diags), bar_check(), fail)).parse_next(input)
}

fn note_leader() -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<NoteLeader>> {
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

fn note_line(diags: &Diagnostics) -> impl FnMut(&mut Input2) -> winnow::Result<Spanned<NoteLine>> {
    move |input| {
        (
            preceded(optional_space, note_leader()),
            terminated(
                combinator::repeat(1.., preceded(space, note(diags))),
                newline(),
            ),
        )
            .with_taken()
            .parse_next(input)
            .map(
                |((leader, notes), tokens): ((Spanned<NoteLeader>, Vec<Spanned<Note>>), Input2)| {
                    let span = tokens.get_span().unwrap();
                    Spanned::new(
                        span,
                        NoteLine {
                            leader: leader.value,
                            notes,
                        },
                    )
                },
            )
    }
}

// TODO: dynamics, dynamic lines

fn consume_one<T>(items: &mut &[T]) {
    if !items.is_empty() {
        *items = &items[1..]
    }
}

fn promote<'s>(lt: &Token1<'s>, t: Pass2) -> Token2<'s> {
    Token::new_spanned(lt.value.raw, lt.span, t)
}

fn promote_and_consume_first<'s>(input: &mut Input2<'_, 's>, t: Pass2) -> Token2<'s> {
    let tok = promote(&input[0], t);
    consume_one(input);
    tok
}

/// Handle the current token, advancing input and appending to out as needed.
fn handle_token<'s>(
    src: &'s str,
    input: &mut Input2<'_, 's>,
    diags: &Diagnostics,
) -> Option<Token2<'s>> {
    let tok = &input[0];
    match &tok.value.t {
        Pass1::Space => Some(promote_and_consume_first(input, Pass2::Space)),
        Pass1::Newline => Some(promote_and_consume_first(input, Pass2::Newline)),
        Pass1::Comment => Some(promote_and_consume_first(input, Pass2::Comment)),
        Pass1::Identifier => {
            // Try to parse as a directive.
            if let Ok(d) = directive(diags).parse_next(input) {
                Some(Token::new_spanned(
                    &src[d.span],
                    d.span,
                    Pass2::Directive(d.value),
                ))
            } else {
                diags.err(code::SYNTAX, tok.span, "unable to parse as directive");
                None
            }
        }
        Pass1::NoteLeader { .. } => {
            if let Ok(x) = note_line(diags).parse_next(input) {
                Some(Token::new_spanned(
                    &src[x.span],
                    x.span,
                    Pass2::NoteLine(x.value),
                ))
            } else {
                diags.err(code::SYNTAX, tok.span, "unable to parse as note line");
                None
            }
        }
        Pass1::DynamicLeader { .. } => None, // TODO
        _ => {
            diags.err(
                code::SYNTAX,
                tok.span,
                format!("unexpected item ({:?})", tok.value.t),
            );
            None
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
            let pr = pitch_or_ratio(&d).parse(input);
            match pr {
                Ok(pr) => {
                    if d.has_errors() {
                        diags = Some(d);
                    } else {
                        p = Some(pr.into_pitch());
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
    let low_tokens = pass1::parse1(src)?;
    let diags = Diagnostics::new();
    let mut input = low_tokens.as_slice();
    let mut out: Vec<Token2> = Vec::new();

    while !input.is_empty() {
        if let Some(tok) = handle_token(src, &mut input, &diags) {
            model::trace(format!("lex pass 2: {tok:?}"));
            out.push(tok);
        } else {
            // TODO: Implement more robust error recovery.
            // Discard the next token and continue.
            consume_one(&mut input);
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
