use crate::parsing::model::{
    Diagnostics, Directive, Param, ParamValue, PitchOrRatio, Span, Spanned, SpannedToken, code,
};
use crate::parsing::pass1::Token1;
use crate::parsing::{model, pass1};
use crate::pitch::{Factor, Pitch};
use crate::to_anyhow;
use anyhow::anyhow;
use num_rational::Ratio;
use std::fmt::Debug;
use winnow::Parser;
use winnow::combinator::{alt, delimited, opt, peek, preceded, separated, terminated};
use winnow::token::{one_of, take_while};

#[derive(Debug, Clone)]
pub enum Token2 {
    // Space, comments
    Space,
    Newline,
    Comment,
    Directive(Directive),
}

fn optional_space(input: &mut &[SpannedToken<Token1>]) -> winnow::Result<()> {
    opt(one_of(|x: SpannedToken<Token1>| {
        matches!(x.t, Token1::Space)
    }))
    .parse_next(input)
    .map(|_| ())
}

fn optional_space_or_newline(input: &mut &[SpannedToken<Token1>]) -> winnow::Result<()> {
    take_while(0.., |x: SpannedToken<Token1>| {
        matches!(x.t, Token1::Space | Token1::Newline | Token1::Comment)
    })
    .parse_next(input)
    .map(|_| ())
}

fn param_separator(input: &mut &[SpannedToken<Token1>]) -> winnow::Result<()> {
    (optional_space, character(','), optional_space_or_newline)
        .parse_next(input)
        .map(|_| ())
}

fn character(
    ch: char,
) -> impl FnMut(&mut &[SpannedToken<Token1>]) -> winnow::Result<(char, usize)> {
    move |input| {
        one_of(|x: SpannedToken<Token1>| x.data.len() == 1 && x.data.starts_with(ch))
            .parse_next(input)
            .map(|x| (ch, x.span.start))
    }
}
fn ratio(
    diags: &Diagnostics,
) -> impl FnMut(&mut &[SpannedToken<Token1>]) -> winnow::Result<(Ratio<u32>, Span)> {
    // Accept this as a ratio and consume the tokens as long as it is syntactically valid. If
    // there are problems report the errors.
    move |input| {
        (
            one_of(|x: SpannedToken<Token1>| matches!(x.t, Token1::RawNumber)),
            opt(preceded(
                character('.'),
                one_of(|x: SpannedToken<Token1>| matches!(x.t, Token1::RawNumber)),
            )),
            opt(preceded(
                character('/'),
                one_of(|x: SpannedToken<Token1>| matches!(x.t, Token1::RawNumber)),
            )),
        )
            .with_taken()
            .parse_next(input)
            .map(|((num_dec_t, num_frac_t, den_t), tokens)| {
                // We already know the numbers can be parsed into u32 from the first lexing pass.
                let span = model::merge_span(tokens);
                let num_dec: u32 = num_dec_t.data.parse().unwrap();
                let (num_frac, scale) = match num_frac_t {
                    None => (0, 1),
                    Some(frac) => {
                        if frac.data.len() > 3 {
                            diags.err(
                                code::NUMBER,
                                frac.span,
                                "a maximum of three decimal places is allowed",
                            );
                            // return any non-zero value to avoid a spurious zero error
                            (1, 10)
                        } else {
                            let v: u32 = frac.data.parse().unwrap();
                            (v, 10u32.pow(frac.data.len() as u32))
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
                    let den: u32 = den_t.data.parse().unwrap();
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

                (Ratio::new(numerator, denominator), span)
            })
    }
}

fn exponent(
    diags: &Diagnostics,
) -> impl FnMut(&mut &[SpannedToken<Token1>]) -> winnow::Result<Factor> {
    // Accept this as an exponent and consume the tokens as long as it is syntactically valid. If
    // there are problems report the errors.
    move |input| {
        (
            opt((
                one_of(|x: SpannedToken<Token1>| matches!(x.t, Token1::RawNumber)),
                opt(preceded(
                    character('/'),
                    one_of(|x: SpannedToken<Token1>| matches!(x.t, Token1::RawNumber)),
                )),
            )),
            preceded(
                character('^'),
                (
                    opt(character('-')),
                    one_of(|x: SpannedToken<Token1>| matches!(x.t, Token1::RawNumber)),
                    preceded(
                        character('|'),
                        one_of(|x: SpannedToken<Token1>| matches!(x.t, Token1::RawNumber)),
                    ),
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
                            num.data.parse().unwrap(),
                            match den {
                                None => 1,
                                Some(den) => den.data.parse().unwrap(),
                            },
                        )
                    }
                };
                let mut exp_num: i32 = exp_num_t.data.parse().unwrap();
                let exp_den = exp_den_t.data.parse().unwrap();
                if let Some((_, offset)) = sign_t {
                    span_start = offset;
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

fn factor(
    diags: &Diagnostics,
) -> impl FnMut(&mut &[SpannedToken<Token1>]) -> winnow::Result<Factor> {
    move |input| alt((exponent(diags), ratio(diags).map(|x| Factor::from(x.0)))).parse_next(input)
}

fn pitch_or_ratio(
    diags: &Diagnostics,
) -> impl FnMut(&mut &[SpannedToken<Token1>]) -> winnow::Result<PitchOrRatio> {
    move |input| {
        let as_ratio = peek(ratio(diags)).parse_next(input);
        preceded(
            opt(character('*')),
            separated(1.., factor(diags), character('*')),
        )
        .with_taken()
        .parse_next(input)
        .map(|(factors, tokens)| {
            let span = model::merge_span(tokens);
            let p = Pitch::new(factors);
            if let Ok((r, r_span)) = as_ratio {
                if r_span == span {
                    // This pitch is parseable as a ratio. Treat it as a ration, and allow the
                    // semantic layer to upgrade it to a pitch later if needed.
                    PitchOrRatio::Ratio((r, p))
                } else {
                    PitchOrRatio::Pitch(p)
                }
            } else {
                PitchOrRatio::Pitch(p)
            }
        })
    }
}

fn identifier<'s>(input: &mut &[SpannedToken<'s, Token1>]) -> winnow::Result<Spanned<String>> {
    one_of(|x: SpannedToken<Token1>| matches!(x.t, Token1::Identifier))
        .parse_next(input)
        .map(|t| Spanned::new(t.span, t.data))
}

fn string(
    _diags: &Diagnostics,
) -> impl FnMut(&mut &[SpannedToken<Token1>]) -> winnow::Result<String> {
    move |input| {
        one_of(|x: SpannedToken<Token1>| matches!(x.t, Token1::RawString))
            .parse_next(input)
            .map(|tok| {
                if tok.data.len() < 2 {
                    unreachable!("length of raw string token < 2");
                }
                // We already know this string starts and ends with `"`.
                let data = &tok.data[1..tok.data.len() - 1];
                data.chars().filter(|c| *c != '\\').collect()
            })
    }
}

fn param_value(
    diags: &Diagnostics,
) -> impl FnMut(&mut &[SpannedToken<Token1>]) -> winnow::Result<Spanned<ParamValue>> {
    move |input| {
        alt((
            string(diags).map(ParamValue::String),
            pitch_or_ratio(diags).map(ParamValue::PitchOrRatio),
        ))
        .with_taken()
        .parse_next(input)
        .map(|(value, tokens)| Spanned::new(model::merge_span(tokens), value))
    }
}

fn param(diags: &Diagnostics) -> impl FnMut(&mut &[SpannedToken<Token1>]) -> winnow::Result<Param> {
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

fn directive(
    diags: &Diagnostics,
) -> impl FnMut(&mut &[SpannedToken<Token1>]) -> winnow::Result<Directive> {
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
            .parse_next(input)
            .map(|(name, params): (Spanned<String>, Vec<Param>)| Directive { name, params })
    }
}

fn consume_one<T>(items: &mut &[T]) {
    if !items.is_empty() {
        *items = &items[1..]
    }
}

fn promote<'s>(lt: &SpannedToken<'s, Token1>, t: Token2) -> SpannedToken<'s, Token2> {
    SpannedToken {
        span: lt.span,
        data: lt.data,
        t,
    }
}

fn promote_and_consume_first<'s>(
    input: &mut &[SpannedToken<'s, Token1>],
    t: Token2,
) -> SpannedToken<'s, Token2> {
    let tok = promote(&input[0], t);
    consume_one(input);
    tok
}

/// Handle the current token, advancing input and appending to out as needed.
fn handle_top_token<'s>(
    src: &'s str,
    input: &mut &[SpannedToken<'s, Token1>],
    diags: &Diagnostics,
    out: &mut Vec<SpannedToken<'s, Token2>>,
) {
    // Look at the token. Some tokens can be immediately handled. Others indicate a branch
    // for further processing.
    let mut parse_directive = false;
    let tok = &input[0];
    // At the top level, all we're allowed to have is directives, which consist of identifiers,
    // numbers, pitches, strings, and the syntactic punctation for identifiers. Anything else
    // (other than spaces and comments) is an error.
    let out_tok = match &tok.t {
        Token1::Space => Some(promote_and_consume_first(input, Token2::Space)),
        Token1::Newline => Some(promote_and_consume_first(input, Token2::Newline)),
        Token1::Comment => Some(promote_and_consume_first(input, Token2::Comment)),
        Token1::Identifier => {
            parse_directive = true;
            None
        }
        _ => None,
    };
    if let Some(tok) = out_tok {
        model::trace(format!("lex pass 2: {tok:?}"));
        out.push(tok);
        return;
    }
    let offset = tok.span.start;
    if !parse_directive {
        diags.err(code::SYNTAX, tok.span, "unexpected item");
        consume_one(input);
        return;
    }

    if let Ok((d, tokens)) = directive(diags).with_taken().parse_next(input) {
        let span = model::merge_span(tokens);
        let t = Token2::Directive(d);
        let data = &src[span];
        out.push(SpannedToken { span, data, t })
    } else {
        diags.err(code::SYNTAX, offset..offset + 1, "unable to parse as pitch");
        consume_one(input);
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
        anyhow!("unable to parse pitch")
    };
    Err(anyhow!("{s}: {err}"))
}

pub fn parse2<'s>(src: &'s str) -> Result<Vec<SpannedToken<'s, Token2>>, Diagnostics> {
    enum LexState {
        Top,
        _PartLine,
        _NoteLine,
    }
    let mut state = LexState::Top;

    let low_tokens = pass1::parse1(src)?;
    let diags = Diagnostics::new();
    let mut input = low_tokens.as_slice();
    let mut out: Vec<SpannedToken<Token2>> = Vec::new();

    let mut next_at_bol = true; // whether next token as at beginning of line
    while !input.is_empty() {
        let tok = &input[0];
        let at_bol = next_at_bol;
        // See if the next token will still be at the beginning of a line.
        next_at_bol =
            matches!(&tok.t, Token1::Newline) || (next_at_bol && matches!(&tok.t, Token1::Space));
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

#[cfg(test)]
mod tests;
