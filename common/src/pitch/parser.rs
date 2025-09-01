use crate::pitch::{Factor, Pitch};
use std::fmt::{Display, Formatter};
use winnow::ascii::dec_int;
use winnow::combinator::{alt, cut_err, opt, preceded, separated};
use winnow::error::{ContextError, StrContext, StrContextValue};
use winnow::stream::AsChar;
use winnow::token::take_while;
use winnow::{ModalParser, Parser};

#[derive(Debug)]
struct DetailedError {
    msg: String,
}
impl Display for DetailedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl std::error::Error for DetailedError {}

fn digits<'s>() -> impl ModalParser<&'s str, u32, ContextError<StrContext>> {
    at_most_n_digits(0)
}

fn at_most_n_digits<'s>(
    max_digits: usize,
) -> impl ModalParser<&'s str, u32, ContextError<StrContext>> {
    move |input: &mut &'s str| {
        take_while(1.., AsChar::is_dec_digit)
            .try_map(|s: &str| {
                if max_digits == 0 || s.len() <= max_digits {
                    Ok(s)
                } else {
                    Err(DetailedError {
                        msg: format!("maximum allowed digits is {max_digits}"),
                    })
                }
            })
            .context(StrContext::Label("number digit length check"))
            .try_map(str::parse::<u32>)
            .context(StrContext::Label("parse number to u32"))
            .parse_next(input)
    }
}

/// Recognize a ratio: `a[.nnn]/b`
fn ratio<'s>() -> impl ModalParser<&'s str, Factor, ContextError<StrContext>> {
    const MAX_DECIMALS: usize = 3;
    (
        digits().context(StrContext::Label("ratio numerator")),
        opt(preceded(
            ".",
            cut_err(
                at_most_n_digits(MAX_DECIMALS)
                    .with_taken()
                    .context(StrContext::Label("ratio numerator decimals")),
            ),
        )),
        opt(preceded(
            "/",
            cut_err(digits()).context(StrContext::Label("ratio denominator")),
        )),
    )
        .context(StrContext::Label("ratio"))
        .try_map(|(num_dec, frac, denominator)| {
            let (num_frac, scale) = match frac {
                None => (0, 1),
                Some((v, s)) => (v, 10u32.pow(s.len() as u32)),
            };
            let numerator = num_dec * scale + num_frac;
            let denominator = denominator.unwrap_or(1) * scale;
            Factor::new(numerator, denominator, 1, 1)
                .map_err(|e| DetailedError { msg: e.to_string() })
        })
}

fn exponent<'s>() -> impl ModalParser<&'s str, Factor, ContextError<StrContext>> {
    (
        opt((
            digits().context(StrContext::Label("base numerator")),
            opt(preceded(
                "/",
                cut_err(digits()).context(StrContext::Label("base denominator")),
            )),
        )),
        preceded(
            "^",
            cut_err((
                dec_int.context(StrContext::Label("exponent numerator")),
                preceded(
                    "|",
                    dec_int
                        .context(StrContext::Label("exponent denominator"))
                        .verify(|x| *x > 0)
                        .context(StrContext::Expected(StrContextValue::Description(
                            "exponent denominator must be > 0",
                        ))),
                ),
            )),
        ),
    )
        .context(StrContext::Label("exponent"))
        .try_map(|(base, (exp_num, exp_den))| {
            let (base_num, base_den) = match base {
                Some((num, Some(den))) => (num, den),
                Some((num, None)) => (num, 1),
                None => (2, 1),
            };
            Factor::new(base_num, base_den, exp_num, exp_den)
                .map_err(|e| DetailedError { msg: e.to_string() })
        })
}

fn factor<'s>() -> impl ModalParser<&'s str, Factor, ContextError<StrContext>> {
    alt((exponent(), ratio())).context(StrContext::Label("factor"))
}

pub(super) fn pitch<'s>() -> impl ModalParser<&'s str, Pitch, ContextError<StrContext>> {
    preceded(opt("*"), separated(1.., factor(), "*"))
        .context(StrContext::Label("factors"))
        .map(Pitch::new)
        .context(StrContext::Label("pitch"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn test_digits() -> anyhow::Result<()> {
        let mut input = "0123abc";
        let x = digits().parse_next(&mut input).unwrap();
        assert_eq!(input, "abc");
        assert_eq!(x, 123);
        let e = digits().parse_next(&mut input).unwrap_err();
        assert!(e.to_string().contains("digit length check"));

        let mut input = "012345abc";
        let e = at_most_n_digits(3).parse_next(&mut input).unwrap_err();
        assert!(e.to_string().contains("digit length check"));
        assert!(e.to_string().contains("maximum allowed digits is 3"));

        let mut input = "0123452348723948729387492387abc";
        let e = digits().parse_next(&mut input).unwrap_err();
        assert!(e.to_string().contains("parse number to u32"));
        assert_eq!(input, "0123452348723948729387492387abc");
        Ok(())
    }

    #[test]
    fn test_ratio() -> anyhow::Result<()> {
        let mut input = "2/3*2.1/3*";
        let f = ratio()
            .parse_next(&mut input)
            .map_err(|e| anyhow!("{e:?}"))?;
        assert_eq!(input, "*2.1/3*");
        assert_eq!(f, Factor::new(2, 3, 1, 1)?);

        let mut input = "264";
        let f = ratio()
            .parse_next(&mut input)
            .map_err(|e| anyhow!("{e:?}"))?;
        assert!(input.is_empty());
        assert_eq!(f, Factor::new(264, 1, 1, 1)?);

        let mut input = "2.1/3";
        let f = ratio()
            .parse_next(&mut input)
            .map_err(|e| anyhow!("{e:?}"))?;
        assert!(input.is_empty());
        assert_eq!(f, Factor::new(7, 10, 1, 1)?);

        let mut input = "2.001/3";
        let f = ratio()
            .parse_next(&mut input)
            .map_err(|e| anyhow!("{e:?}"))?;
        assert!(input.is_empty());
        assert_eq!(f, Factor::new(667, 1000, 1, 1)?);

        let mut input = "2.0001/3";
        let e = ratio().parse_next(&mut input).unwrap_err();
        assert!(e.to_string().contains("numerator decimals"));
        let mut input = "0/4";
        let e = ratio().parse_next(&mut input).unwrap_err();
        assert!(e.to_string().contains("zero may not"));
        let mut input = "5/0";
        let e = ratio().parse_next(&mut input).unwrap_err();
        assert!(e.to_string().contains("zero may not"));
        Ok(())
    }

    #[test]
    fn test_exponent() -> anyhow::Result<()> {
        let mut input = "^1|31*";
        let f = exponent()
            .parse_next(&mut input)
            .map_err(|e| anyhow!("{e:?}"))?;
        assert_eq!(input, "*");
        assert_eq!(f, Factor::new(2, 1, 1, 31)?);

        let mut input = "3^2|17";
        let f = exponent()
            .parse_next(&mut input)
            .map_err(|e| anyhow!("{e:?}"))?;
        assert!(input.is_empty());
        assert_eq!(f, Factor::new(3, 1, 2, 17)?);

        let mut input = "3/2^-9|12";
        let f = exponent()
            .parse_next(&mut input)
            .map_err(|e| anyhow!("{e:?}"))?;
        assert!(input.is_empty());
        assert_eq!(f, Factor::new(3, 2, -9, 12)?);

        let mut input = "3/2^0|12";
        let f = exponent()
            .parse_next(&mut input)
            .map_err(|e| anyhow!("{e:?}"))?;
        assert!(input.is_empty());
        assert_eq!(f, Factor::new(3, 2, 0, 12)?);

        let mut input = "^5|0";
        let e = exponent().parse_next(&mut input).unwrap_err();
        assert!(e.to_string().contains("exponent denominator"));
        Ok(())
    }
}
