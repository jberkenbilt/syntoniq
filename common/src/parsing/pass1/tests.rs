use super::*;
use crate::parsing::diagnostics::Diagnostic;
use crate::to_anyhow;

/// Test first stage parsers that work with strings
macro_rules! make_parser1 {
    ($f:ident, $p:ident) => {
        fn $f<'s>(src: &'s str) -> Result<(Token1<'s>, &'s str), Diagnostics> {
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

make_parser1!(parse_raw_number, number);
make_parser1!(parse_string_literal, string_literal);

#[test]
fn test_raw_number() -> anyhow::Result<()> {
    assert!(!parse_raw_number("potato").unwrap_err().has_errors());

    let (s, rest) = parse_raw_number("16059q").map_err(to_anyhow)?;
    assert_eq!(s.value.raw, "16059");
    assert_eq!(rest, "q");

    let e = parse_raw_number("14159265358979323846264w")
        .unwrap_err()
        .get_all();
    assert_eq!(
        e,
        [Diagnostic::new(
            code::LEXICAL,
            0..23,
            "while parsing number: number too large to fit in target type"
        )]
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
    assert_eq!(s.value.raw, r#""string with \"π\" and \\""#);
    assert_eq!(rest, "w");
    let s = Pass1::get_string(&s).unwrap();
    assert_eq!(s, Spanned::new(1..26, r#"string with "π" and \"#));

    let e = parse_string_literal("\"invalid π \\quoted and\\🥔\n in the middle\"")
        .unwrap_err()
        .get_all();
    assert_eq!(
        e,
        [
            Diagnostic::new(code::LEXICAL, 13..14, "invalid quoted character"),
            Diagnostic::new(code::LEXICAL, 24..28, "invalid quoted character"),
            Diagnostic::new(
                code::LEXICAL,
                28..29,
                "string may not contain newline characters"
            ),
        ]
    );

    Ok(())
}
