use super::*;
use winnow::combinator::{delimited, terminated};

pub(super) fn raw_identifier<'s>() -> impl Parser<Inp<'s>, &'s str, CErr> {
    (
        take_while(1, |c: char| AsChar::is_alpha(c)),
        take_while(0.., |c: char| AsChar::is_alphanum(c) || c == '_'),
    )
        .take()
        .context(StrContext::Label("identifier"))
}

pub(super) fn raw_number<'s>(
    diags: &Diagnostics,
) -> impl FnMut(&mut Inp<'s>) -> winnow::Result<&'s str> {
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

pub(super) fn string_literal<'s>(
    diags: &Diagnostics,
) -> impl FnMut(&mut Inp<'s>) -> winnow::Result<&'s str> {
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

fn optional_space(input: &mut &[SpannedToken<LowToken>]) -> winnow::Result<()> {
    opt(one_of(|x: SpannedToken<LowToken>| {
        matches!(x.t, LowToken::Space)
    }))
    .parse_next(input)
    .map(|_| ())
}

fn optional_space_or_newline(input: &mut &[SpannedToken<LowToken>]) -> winnow::Result<()> {
    take_while(0.., |x: SpannedToken<LowToken>| {
        matches!(x.t, LowToken::Space | LowToken::Newline | LowToken::Comment)
    })
    .parse_next(input)
    .map(|_| ())
}

fn param_separator(input: &mut &[SpannedToken<LowToken>]) -> winnow::Result<()> {
    (optional_space, character(','), optional_space_or_newline)
        .parse_next(input)
        .map(|_| ())
}

fn character(
    ch: char,
) -> impl FnMut(&mut &[SpannedToken<LowToken>]) -> winnow::Result<(char, usize)> {
    move |input| {
        one_of(|x: SpannedToken<LowToken>| x.data.len() == 1 && x.data.starts_with(ch))
            .parse_next(input)
            .map(|x| (ch, x.span.start))
    }
}
fn ratio(
    diags: &Diagnostics,
) -> impl FnMut(&mut &[SpannedToken<LowToken>]) -> winnow::Result<(Ratio<u32>, Span)> {
    // Accept this as a ratio and consume the tokens as long as it is syntactically valid. If
    // there are problems report the errors.
    move |input| {
        (
            one_of(|x: SpannedToken<LowToken>| matches!(x.t, LowToken::RawNumber)),
            opt(preceded(
                character('.'),
                one_of(|x: SpannedToken<LowToken>| matches!(x.t, LowToken::RawNumber)),
            )),
            opt(preceded(
                character('/'),
                one_of(|x: SpannedToken<LowToken>| matches!(x.t, LowToken::RawNumber)),
            )),
        )
            .with_taken()
            .parse_next(input)
            .map(|((num_dec_t, num_frac_t, den_t), tokens)| {
                // We already know the numbers can be parsed into u32 from the first lexing pass.
                let span = merge_span(tokens);
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
) -> impl FnMut(&mut &[SpannedToken<LowToken>]) -> winnow::Result<Factor> {
    // Accept this as an exponent and consume the tokens as long as it is syntactically valid. If
    // there are problems report the errors.
    move |input| {
        (
            opt((
                one_of(|x: SpannedToken<LowToken>| matches!(x.t, LowToken::RawNumber)),
                opt(preceded(
                    character('/'),
                    one_of(|x: SpannedToken<LowToken>| matches!(x.t, LowToken::RawNumber)),
                )),
            )),
            preceded(
                character('^'),
                (
                    opt(character('-')),
                    one_of(|x: SpannedToken<LowToken>| matches!(x.t, LowToken::RawNumber)),
                    preceded(
                        character('|'),
                        one_of(|x: SpannedToken<LowToken>| matches!(x.t, LowToken::RawNumber)),
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
) -> impl FnMut(&mut &[SpannedToken<LowToken>]) -> winnow::Result<Factor> {
    move |input| alt((exponent(diags), ratio(diags).map(|x| Factor::from(x.0)))).parse_next(input)
}

pub(super) fn pitch_or_ratio(
    diags: &Diagnostics,
) -> impl FnMut(&mut &[SpannedToken<LowToken>]) -> winnow::Result<PitchOrRatio> {
    move |input| {
        let as_ratio = peek(ratio(diags)).parse_next(input);
        preceded(
            opt(character('*')),
            separated(1.., factor(diags), character('*')),
        )
        .with_taken()
        .parse_next(input)
        .map(|(factors, tokens)| {
            let span = merge_span(tokens);
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

pub(super) fn identifier<'s>(
    input: &mut &[SpannedToken<'s, LowToken>],
) -> winnow::Result<Spanned<String>> {
    one_of(|x: SpannedToken<LowToken>| matches!(x.t, LowToken::Identifier))
        .parse_next(input)
        .map(|t| Spanned::new(t.span, t.data))
}

pub(super) fn string(
    _diags: &Diagnostics,
) -> impl FnMut(&mut &[SpannedToken<LowToken>]) -> winnow::Result<String> {
    move |input| {
        one_of(|x: SpannedToken<LowToken>| matches!(x.t, LowToken::RawString))
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

pub(super) fn param_value(
    diags: &Diagnostics,
) -> impl FnMut(&mut &[SpannedToken<LowToken>]) -> winnow::Result<Spanned<ParamValue>> {
    move |input| {
        alt((
            string(diags).map(ParamValue::String),
            pitch_or_ratio(diags).map(ParamValue::PitchOrRatio),
        ))
        .with_taken()
        .parse_next(input)
        .map(|(value, tokens)| Spanned::new(merge_span(tokens), value))
    }
}

pub(super) fn param(
    diags: &Diagnostics,
) -> impl FnMut(&mut &[SpannedToken<LowToken>]) -> winnow::Result<Param> {
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

pub(super) fn directive(
    diags: &Diagnostics,
) -> impl FnMut(&mut &[SpannedToken<LowToken>]) -> winnow::Result<Directive> {
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

#[cfg(test)]
mod tests;
