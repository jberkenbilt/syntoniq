use super::*;
use crate::parsing::diagnostics::Diagnostic;
use crate::to_anyhow;

// Trying to make this general is very hard because of all the different lifetime bounds on
// parsers, so use macros since this is test code.

/// Test first stage parsers that work with strings
macro_rules! make_parser1 {
    ($f:ident, $p:ident) => {
        fn $f<'s>(src: &'s str) -> Result<(&'s str, &'s str), Diagnostics> {
            let mut input = LocatingSlice::new(src);
            let diags = Diagnostics::new();
            let r = $p(&diags).parse_next(&mut input);
            if diags.has_errors() {
                return Err(diags);
            }
            let rest = *input.as_ref();
            r.map(|r| (r, rest)).map_err(|_| diags)
        }
    };
}

/// Test second stage parsers that work with LowTokens
macro_rules! make_parser2 {
    ($f:ident, $p:ident, $r:ty) => {
        fn $f(s: &str) -> Result<($r, &str), Diagnostics> {
            let tokens = lex_pass1(s)?;
            let mut input = tokens.as_slice();
            let diags = Diagnostics::new();
            let r = $p(&diags).parse_next(&mut input);
            if diags.has_errors() {
                return Err(diags);
            }
            let rest = if input.is_empty() {
                ""
            } else {
                let span = merge_span(input);
                &s[span]
            };
            r.map(|r| (r, rest)).map_err(|_| diags)
        }
    };
}

make_parser1!(parse_raw_number, raw_number);
make_parser1!(parse_string_literal, string_literal);

make_parser2!(parse_ratio, ratio, (Ratio<u32>, Span));
make_parser2!(parse_exponent, exponent, Factor);
make_parser2!(parse_pitch, pitch_or_ratio, PitchOrRatio);

#[test]
fn test_raw_number() -> anyhow::Result<()> {
    assert!(!parse_raw_number("potato").unwrap_err().has_errors());

    let (s, rest) = parse_raw_number("16059q").map_err(to_anyhow)?;
    assert_eq!(s, "16059");
    assert_eq!(rest, "q");

    let e = parse_raw_number("14159265358979323846264w")
        .unwrap_err()
        .get_all();
    assert_eq!(
        e,
        [Diagnostic {
            code: code::LEXICAL,
            span: (0..23).into(),
            // Part of this is a rust error which may change (but not likely)
            message: "while parsing number: number too large to fit in target type".to_string(),
        }]
    );
    Ok(())
}

#[test]
fn test_string_literal() -> anyhow::Result<()> {
    let x = parse_string_literal("potato");
    assert!(!x.unwrap_err().has_errors());
    let x = parse_string_literal("\"salad");
    assert!(!x.unwrap_err().has_errors());

    let (s, rest) = parse_string_literal(r#""string with \"π\" and \\"w"#).map_err(to_anyhow)?;
    assert_eq!(s, r#""string with \"π\" and \\""#);
    assert_eq!(rest, "w");

    let e = parse_string_literal("\"invalid π \\quoted and\\🥔\n in the middle\"")
        .unwrap_err()
        .get_all();
    assert_eq!(
        e,
        [
            Diagnostic {
                code: code::LEXICAL,
                span: (13..14).into(),
                message: "invalid quoted character".to_string(),
            },
            Diagnostic {
                code: code::LEXICAL,
                span: (24..28).into(),
                message: "invalid quoted character".to_string(),
            },
            Diagnostic {
                code: code::LEXICAL,
                span: (28..29).into(),
                message: "string may not contain newline characters".to_string(),
            }
        ]
    );

    Ok(())
}

#[test]
fn test_ratio() -> anyhow::Result<()> {
    // Not a ratio, no errors found while scanning
    assert!(!parse_ratio("potato").unwrap_err().has_errors());

    let (f, rest) = parse_ratio("2/3*2.1/3*").map_err(to_anyhow)?;
    assert_eq!(rest, "*2.1/3*");
    assert_eq!(f.0, Ratio::new(2, 3));

    let (f, rest) = parse_ratio("264").map_err(to_anyhow)?;
    assert!(rest.is_empty());
    assert_eq!(f.0, Ratio::new(264, 1));

    let (f, rest) = parse_ratio("2.1/3").map_err(to_anyhow)?;
    assert!(rest.is_empty());
    assert_eq!(f.0, Ratio::new(7, 10));

    let (f, rest) = parse_ratio("3.14").map_err(to_anyhow)?;
    assert!(rest.is_empty());
    assert_eq!(f.0, Ratio::new(157, 50));

    let (f, rest) = parse_ratio("2.001/3").map_err(to_anyhow)?;
    assert!(rest.is_empty());
    assert_eq!(f.0, Ratio::new(667, 1000));

    let (f, rest) = parse_ratio("22/7").map_err(to_anyhow)?;
    assert!(rest.is_empty());
    assert_eq!(f.0, Ratio::new(22, 7));

    let e = parse_ratio("2.0001/3").unwrap_err().get_all();
    assert_eq!(
        e,
        [Diagnostic {
            code: code::NUMBER,
            span: (2..6).into(),
            message: "a maximum of three decimal places is allowed".to_string(),
        }]
    );

    let e = parse_ratio("123456789.001").unwrap_err().get_all();
    assert_eq!(
        e,
        [Diagnostic {
            code: code::NUMBER,
            span: (0..9).into(),
            message: "insufficient precision for numerator".to_string(),
        }]
    );

    let e = parse_ratio("1.001/123456789").unwrap_err().get_all();
    assert_eq!(
        e,
        [Diagnostic {
            code: code::NUMBER,
            span: (6..15).into(),
            message: "insufficient precision for denominator".to_string(),
        }]
    );

    let e = parse_ratio("0/0").unwrap_err().get_all();
    assert_eq!(
        e,
        [
            Diagnostic {
                code: code::NUMBER,
                span: (0..1).into(),
                message: "zero not allowed as numerator".to_string(),
            },
            Diagnostic {
                code: code::NUMBER,
                span: (2..3).into(),
                message: "zero not allowed as denominator".to_string(),
            }
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
        [Diagnostic {
            code: "E1004 pitch error",
            span: (1..4).into(),
            message: "zero may not appear anywhere in base or in exponent denominator".to_string()
        }]
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
