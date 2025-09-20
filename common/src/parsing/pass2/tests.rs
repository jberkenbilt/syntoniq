use super::*;
use crate::parsing::diagnostics::Diagnostic;
use crate::parsing::pass1::parse1;

macro_rules! make_parser2 {
    ($f:ident, $p:ident, $r:ty) => {
        fn $f(s: &str) -> Result<($r, &str), Diagnostics> {
            let tokens = parse1(s)?;
            let mut input = tokens.as_slice();
            let diags = Diagnostics::new();
            let r = $p(&diags).parse_next(&mut input);
            if diags.has_errors() {
                return Err(diags);
            }
            let rest = if input.is_empty() {
                ""
            } else {
                let span = input.get_span().unwrap();
                &s[span]
            };
            r.map(|r| (r, rest)).map_err(|_| diags)
        }
    };
}

make_parser2!(parse_ratio, ratio, Spanned<Ratio<u32>>);
make_parser2!(parse_ratio_or_zero, ratio_or_zero, Spanned<Ratio<u32>>);
make_parser2!(parse_exponent, exponent, Factor);
make_parser2!(parse_pitch, pitch_or_number, PitchOrNumber);
make_parser2!(parse_string, string, Spanned<String>);
make_parser2!(parse_param_kv, param_kv, ParamKV);
make_parser2!(parse_directive, directive, Spanned<Directive>);
make_parser2!(parse_octave, octave, Spanned<i8>);

#[test]
fn for_coverage() {
    // Usually I consider 100% coverage to be a non-goal, but for the parser, it's good to have
    // all error conditions tested. This just exercises some cases that are unreachable in the
    // normal flow for coverage.
    let v: Vec<Token1> = Vec::new();
    let mut s = v.as_slice();
    consume_one(&mut s)
}

#[test]
fn test_ratio() -> anyhow::Result<()> {
    // Not a ratio, no errors found while scanning
    assert!(!parse_ratio("potato").unwrap_err().has_errors());

    let (f, rest) = parse_ratio("2/3*2.1/3*").map_err(to_anyhow)?;
    assert_eq!(rest, "*2.1/3*");
    assert_eq!(f.value, Ratio::new(2, 3));

    let (f, rest) = parse_ratio("264").map_err(to_anyhow)?;
    assert!(rest.is_empty());
    assert_eq!(f.value, Ratio::new(264, 1));

    let (f, rest) = parse_ratio("2.1/3").map_err(to_anyhow)?;
    assert!(rest.is_empty());
    assert_eq!(f.value, Ratio::new(7, 10));

    let (f, rest) = parse_ratio("3.14").map_err(to_anyhow)?;
    assert!(rest.is_empty());
    assert_eq!(f.value, Ratio::new(157, 50));

    let (f, rest) = parse_ratio("2.001/3").map_err(to_anyhow)?;
    assert!(rest.is_empty());
    assert_eq!(f.value, Ratio::new(667, 1000));

    let (f, rest) = parse_ratio("22/7").map_err(to_anyhow)?;
    assert!(rest.is_empty());
    assert_eq!(f.value, Ratio::new(22, 7));

    let (f, rest) = parse_ratio_or_zero("00").map_err(to_anyhow)?;
    assert!(rest.is_empty());
    assert_eq!(f.value, Ratio::new(0, 1));

    let e = parse_ratio("2.0001/3").unwrap_err().get_all();
    assert_eq!(
        e,
        [Diagnostic::new(
            code::NUM_FORMAT,
            2..6,
            "a maximum of three decimal places is allowed"
        )]
    );

    let e = parse_ratio("123456789.001").unwrap_err().get_all();
    assert_eq!(
        e,
        [Diagnostic::new(
            code::NUM_FORMAT,
            0..9,
            "too much precision for numerator"
        )]
    );

    let e = parse_ratio("1.001/123456789").unwrap_err().get_all();
    assert_eq!(
        e,
        [Diagnostic::new(
            code::NUM_FORMAT,
            6..15,
            "too much precision for denominator"
        )]
    );

    let e = parse_ratio("0/0").unwrap_err().get_all();
    assert_eq!(
        e,
        [
            Diagnostic::new(code::NUM_FORMAT, 0..1, "zero not allowed as numerator"),
            Diagnostic::new(code::NUM_FORMAT, 2..3, "zero not allowed as denominator")
        ]
    );

    let e = parse_ratio("0").unwrap_err().get_all();
    assert_eq!(
        e,
        [Diagnostic::new(
            code::NUM_FORMAT,
            0..1,
            "zero not allowed as numerator"
        ),]
    );

    let e = parse_ratio_or_zero("0/0").unwrap_err().get_all();
    assert_eq!(
        e,
        [
            Diagnostic::new(code::NUM_FORMAT, 0..1, "zero not allowed as numerator"),
            Diagnostic::new(code::NUM_FORMAT, 2..3, "zero not allowed as denominator")
        ]
    );

    Ok(())
}

#[test]
fn test_exponent() -> anyhow::Result<()> {
    assert!(!parse_exponent("potato").unwrap_err().has_errors());

    let (f, rest) = parse_exponent("^1|31*").map_err(to_anyhow)?;
    assert_eq!(rest, "*");
    assert_eq!(f, Factor::new(2, 1, 1, 31)?);

    let (f, rest) = parse_exponent("3^2|17").map_err(to_anyhow)?;
    assert!(rest.is_empty());
    assert_eq!(f, Factor::new(3, 1, 2, 17)?);

    let (f, rest) = parse_exponent("3/2^-9|12").map_err(to_anyhow)?;
    assert!(rest.is_empty());
    assert_eq!(f, Factor::new(3, 2, -9, 12)?);

    let (f, rest) = parse_exponent("3/2^0|12").map_err(to_anyhow)?;
    assert!(rest.is_empty());
    assert_eq!(f, Factor::new(3, 2, 0, 12)?);

    let e = parse_exponent("^5|0").unwrap_err().get_all();
    assert_eq!(
        e,
        [Diagnostic::new(
            code::PITCH,
            3..4,
            "zero not allowed as exponent denominator"
        )]
    );

    Ok(())
}

#[test]
fn test_pitch() -> anyhow::Result<()> {
    // Most of `pitch` parsing is in tests for the Pitch struct.
    assert!(!parse_pitch("potato").unwrap_err().has_errors());

    let (p, rest) = parse_pitch("^1|31*2/3z").map_err(to_anyhow)?;
    assert_eq!(rest, "z");
    assert!(p.clone().try_into_int().is_none());
    assert!(p.clone().try_into_ratio().is_none());
    assert_eq!(p.into_pitch().to_string(), "2/3*^1|31");

    let (p, rest) = parse_pitch("22/7z").map_err(to_anyhow)?;
    assert_eq!(rest, "z");
    assert!(p.clone().try_into_int().is_none());
    assert_eq!(p.clone().try_into_ratio().unwrap(), Ratio::new(22, 7));
    assert_eq!(p.into_pitch().to_string(), "22/7");

    let (p, rest) = parse_pitch("12z").map_err(to_anyhow)?;
    assert_eq!(rest, "z");
    assert_eq!(p.clone().try_into_int().unwrap(), 12);
    assert_eq!(p.clone().try_into_ratio().unwrap(), Ratio::new(12, 1));
    assert_eq!(p.into_pitch().to_string(), "12");

    Ok(())
}

#[test]
fn test_string() -> anyhow::Result<()> {
    let (s, rest) = parse_string(r#""π are \"◼\""1"#).map_err(to_anyhow)?;
    assert_eq!(s.value, r#"π are "◼""#);
    assert_eq!(rest, "1");
    Ok(())
}

#[test]
fn test_param() -> anyhow::Result<()> {
    let (s, rest) = parse_param_kv("a=^2|19").map_err(to_anyhow)?;
    assert_eq!(
        s,
        ParamKV {
            key: Spanned::new(0..1, "a"),
            value: Spanned::new(
                2..7,
                ParamValue::PitchOrNumber(PitchOrNumber::Pitch(Pitch::must_parse("^2|19")))
            ),
        }
    );
    assert!(rest.is_empty());

    let (s, rest) = parse_param_kv("potato = \"salad\"!").map_err(to_anyhow)?;
    assert_eq!(
        s,
        ParamKV {
            key: Spanned::new(0..6, "potato"),
            value: Spanned::new(9..16, ParamValue::String("salad".to_string())),
        }
    );
    assert_eq!(rest, "!");

    Ok(())
}

#[test]
fn test_directive() -> anyhow::Result<()> {
    let (d, rest) =
        parse_directive("tune(base_pitch=^2|19 scale=\"17-EDO\")").map_err(to_anyhow)?;
    assert_eq!(
        d.value,
        Directive {
            opening_comment: None,
            name: Spanned::new(0..4, "tune"),
            params: vec![
                Param {
                    kv: ParamKV {
                        key: Spanned::new(5..15, "base_pitch"),
                        value: Spanned::new(
                            16..21,
                            ParamValue::PitchOrNumber(PitchOrNumber::Pitch(Pitch::must_parse(
                                "^2|19"
                            )))
                        ),
                    },
                    comment: None,
                },
                Param {
                    kv: ParamKV {
                        key: Spanned::new(22..27, "scale"),
                        value: Spanned::new(28..36, ParamValue::String("17-EDO".to_string())),
                    },
                    comment: None,
                }
            ],
        }
    );
    assert!(rest.is_empty());

    let (d, rest) = parse_directive(
        r#"function ( ; opening comment
            one   = 1
            two   = 22/7
            three = "π+🥔"
            four  = *3^-2|31*3/2   ; comment
        )"#,
    )
    .map_err(to_anyhow)?;
    assert_eq!(
        d.value,
        Directive {
            opening_comment: Some(Comment {
                content: Spanned::new(11..28, "; opening comment")
            }),
            name: Spanned::new(0..8, "function"),
            params: vec![
                Param {
                    kv: ParamKV {
                        key: Spanned::new(41..44, "one"),
                        value: Spanned::new(
                            49..50,
                            ParamValue::PitchOrNumber(PitchOrNumber::Integer((
                                1,
                                Pitch::must_parse("1")
                            )))
                        ),
                    },
                    comment: None
                },
                Param {
                    kv: ParamKV {
                        key: Spanned::new(63..66, "two"),
                        value: Spanned::new(
                            71..75,
                            ParamValue::PitchOrNumber(PitchOrNumber::Ratio((
                                Ratio::new(22, 7),
                                Pitch::must_parse("22/7")
                            )))
                        ),
                    },
                    comment: None
                },
                Param {
                    kv: ParamKV {
                        key: Spanned::new(88..93, "three"),
                        value: Spanned::new(96..105, ParamValue::String("π+🥔".to_string())),
                    },
                    comment: None
                },
                Param {
                    kv: ParamKV {
                        key: Spanned::new(118..122, "four"),
                        value: Spanned::new(
                            126..138,
                            ParamValue::PitchOrNumber(PitchOrNumber::Pitch(Pitch::must_parse(
                                "0.5*3^29|31"
                            )))
                        ),
                    },
                    comment: Some(Comment {
                        content: Spanned::new(141..150, "; comment"),
                    })
                },
            ],
        }
    );
    assert!(rest.is_empty());

    let e = parse_directive("tune(a=^2|19b=\"<- missing space\")")
        .unwrap_err()
        .get_all();
    assert_eq!(
        e,
        vec![Diagnostic::new(
            code::DIRECTIVE,
            5..12,
            "this parameter must be followed by a space, comment, or newline"
        )]
    );

    Ok(())
}

#[test]
fn test_octave() -> anyhow::Result<()> {
    let (o, _) = parse_octave("'2").map_err(to_anyhow)?;
    assert_eq!(o.span, (0..2).into());
    assert_eq!(o.value, 2);
    let (o, _) = parse_octave("'1").map_err(to_anyhow)?;
    assert_eq!(o.value, 1);
    let (o, _) = parse_octave("'").map_err(to_anyhow)?;
    assert_eq!(o.value, 1);
    let (o, _) = parse_octave(",2").map_err(to_anyhow)?;
    assert_eq!(o.value, -2);
    let (o, _) = parse_octave(",1").map_err(to_anyhow)?;
    assert_eq!(o.value, -1);
    let (o, _) = parse_octave(",").map_err(to_anyhow)?;
    assert_eq!(o.value, -1);
    let e = parse_octave(",128").unwrap_err().get_all();
    assert_eq!(
        e,
        vec![Diagnostic::new(
            code::NOTE,
            1..4,
            "octave count is too large"
        )]
    );
    let e = parse_octave("'0").unwrap_err().get_all();
    assert_eq!(
        e,
        vec![Diagnostic::new(
            code::NOTE,
            1..2,
            "octave count may not be zero"
        )]
    );
    Ok(())
}
