use super::*;
use crate::parsing::diagnostics::Diagnostic;
use crate::parsing::pass1::parse1;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{fs, io};

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
make_parser2!(parse_pitch, pitch_or_ratio, PitchOrRatio);
make_parser2!(parse_string, string, Spanned<String>);
make_parser2!(parse_param, param, Param);
make_parser2!(parse_directive, directive, Spanned<Directive>);
make_parser2!(parse_octave, octave, Spanned<i8>);

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
            code::NUMBER,
            2..6,
            "a maximum of three decimal places is allowed"
        )]
    );

    let e = parse_ratio("123456789.001").unwrap_err().get_all();
    assert_eq!(
        e,
        [Diagnostic::new(
            code::NUMBER,
            0..9,
            "insufficient precision for numerator"
        )]
    );

    let e = parse_ratio("1.001/123456789").unwrap_err().get_all();
    assert_eq!(
        e,
        [Diagnostic::new(
            code::NUMBER,
            6..15,
            "insufficient precision for denominator"
        )]
    );

    let e = parse_ratio("0/0").unwrap_err().get_all();
    assert_eq!(
        e,
        [
            Diagnostic::new(code::NUMBER, 0..1, "zero not allowed as numerator"),
            Diagnostic::new(code::NUMBER, 2..3, "zero not allowed as denominator")
        ]
    );

    let e = parse_ratio("0").unwrap_err().get_all();
    assert_eq!(
        e,
        [Diagnostic::new(
            code::NUMBER,
            0..1,
            "zero not allowed as numerator"
        ),]
    );

    let e = parse_ratio_or_zero("0/0").unwrap_err().get_all();
    assert_eq!(
        e,
        [
            Diagnostic::new(code::NUMBER, 0..1, "zero not allowed as numerator"),
            Diagnostic::new(code::NUMBER, 2..3, "zero not allowed as denominator")
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
            "E1004 pitch error",
            1..4,
            "zero may not appear anywhere in base or in exponent denominator"
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
    assert!(p.clone().try_into_ratio().is_none());
    assert_eq!(p.into_pitch().to_string(), "2/3*^1|31");

    let (p, rest) = parse_pitch("22/7z").map_err(to_anyhow)?;
    assert_eq!(rest, "z");
    assert_eq!(p.clone().try_into_ratio().unwrap(), Ratio::new(22, 7));
    assert_eq!(p.into_pitch().to_string(), "22/7");

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
    let (s, rest) = parse_param("a=^2|19").map_err(to_anyhow)?;
    assert_eq!(
        s,
        Param {
            key: Spanned::new(0..1, "a"),
            value: Spanned::new(
                2..7,
                ParamValue::PitchOrRatio(PitchOrRatio::Pitch(Pitch::must_parse("^2|19")))
            ),
        }
    );
    assert!(rest.is_empty());

    let (s, rest) = parse_param("potato = \"salad\"!").map_err(to_anyhow)?;
    assert_eq!(
        s,
        Param {
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
        parse_directive("tune(base_pitch=^2|19, scale=\"17-EDO\")").map_err(to_anyhow)?;
    assert_eq!(
        d.value,
        Directive {
            name: Spanned::new(0..4, "tune"),
            params: vec![
                Param {
                    key: Spanned::new(5..15, "base_pitch"),
                    value: Spanned::new(
                        16..21,
                        ParamValue::PitchOrRatio(PitchOrRatio::Pitch(Pitch::must_parse("^2|19")))
                    ),
                },
                Param {
                    key: Spanned::new(23..28, "scale"),
                    value: Spanned::new(29..37, ParamValue::String("17-EDO".to_string())),
                }
            ],
        }
    );
    assert!(rest.is_empty());

    let (d, rest) = parse_directive(
        r#"function (
            one   = 1 ,
            two   = 22/7 ,
            three = "π+🥔" ,
            four  = *3^-2|31*3/2 ,  ; comment
        )"#,
    )
    .map_err(to_anyhow)?;
    assert_eq!(
        d.value,
        Directive {
            name: Spanned::new(0..8, "function"),
            params: vec![
                Param {
                    key: Spanned::new(23..26, "one"),
                    value: Spanned::new(
                        31..32,
                        ParamValue::PitchOrRatio(PitchOrRatio::Ratio((
                            Ratio::new(1, 1),
                            Pitch::must_parse("1")
                        )))
                    ),
                },
                Param {
                    key: Spanned::new(47..50, "two"),
                    value: Spanned::new(
                        55..59,
                        ParamValue::PitchOrRatio(PitchOrRatio::Ratio((
                            Ratio::new(22, 7),
                            Pitch::must_parse("22/7")
                        )))
                    ),
                },
                Param {
                    key: Spanned::new(74..79, "three"),
                    value: Spanned::new(82..91, ParamValue::String("π+🥔".to_string())),
                },
                Param {
                    key: Spanned::new(106..110, "four"),
                    value: Spanned::new(
                        114..126,
                        ParamValue::PitchOrRatio(PitchOrRatio::Pitch(Pitch::must_parse(
                            "0.5*3^29|31"
                        )))
                    ),
                }
            ],
        }
    );
    assert!(rest.is_empty());

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
            code::SYNTAX,
            1..4,
            "octave count is too large"
        )]
    );
    let e = parse_octave("'0").unwrap_err().get_all();
    assert_eq!(
        e,
        vec![Diagnostic::new(
            code::SYNTAX,
            1..2,
            "octave count may not be zero"
        )]
    );
    Ok(())
}

fn get_stq_files(dir: impl AsRef<Path>) -> anyhow::Result<Vec<PathBuf>> {
    let paths = fs::read_dir(dir)?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.display().to_string().ends_with(".stq") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    Ok(paths)
}

#[test]
fn test_pass2() -> anyhow::Result<()> {
    // TODO: HERE: generate tests and carefully validate or hand-generate output and errors.

    // This is designed to fail if anything failed but to run all the tests and produce useful
    // output for analysis.
    let paths = get_stq_files("parsing-tests/pass2")?;
    let mut errors = Vec::<anyhow::Error>::new();
    for p in paths {
        let out = p.to_str().unwrap().replace(".stq", ".json");
        let in_data = String::from_utf8(fs::read(&p)?)?;
        let out_value: serde_json::Value = serde_json::from_reader(File::open(&out)?)?;
        let r = parse2(&in_data);
        let in_as_value = serde_json::to_string(&r)?;
        let in_value: serde_json::Value = serde_json::from_str(&in_as_value)?;
        if in_value == out_value {
            eprintln!("{}: PASS", p.display());
        } else {
            // Generate output with ./target/debug/tokenize --json
            eprintln!("------ {} ------", p.display());
            eprintln!("------ ACTUAL ------");
            serde_json::to_writer_pretty(io::stderr(), &r)?;
            eprintln!("------ EXPECTED ------");
            serde_json::to_writer_pretty(io::stderr(), &out_value)?;
            errors.push(anyhow!("{}: FAIL", p.display()));
        }
    }
    if !errors.is_empty() {
        for e in errors {
            eprintln!("ERROR: {e}");
        }
        panic!("there were errors");
    }
    Ok(())
}
