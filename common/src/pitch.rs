use crate::to_anyhow;
use anyhow::bail;
use num_rational::Ratio;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, de};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use winnow::Parser;

mod parser;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pitch {
    factors: Vec<Factor>,
}
impl PartialOrd for Pitch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Pitch {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            Ordering::Equal
        } else {
            let freq1 = self.as_float();
            let freq2 = other.as_float();
            if freq1 < freq2 {
                Ordering::Less
            } else if freq1 > freq2 {
                Ordering::Greater
            } else if freq1 == freq2 {
                // If this is reached, there's a bug in canonicalization
                log::warn!("{self} and {other} have the same frequency but are not equal");
                Ordering::Equal
            } else {
                // Should not be possible...this would mean infinity or NAN from as_float().
                panic!("{self} and {other} are not comparable");
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Factor {
    base: Ratio<u32>,
    exp: Ratio<i32>,
}

impl Display for Factor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fn write_frac(f: &mut Formatter<'_>, num: u32, den: u32) -> fmt::Result {
            write!(f, "{num}")?;
            if den != 1 {
                write!(f, "/{den}")?;
            }
            Ok(())
        }

        let num = *self.base.numer();
        let den = *self.base.denom();
        if self.exp == Ratio::from_integer(1) {
            // If this is a large enough number to be likely to be a frequency and has a fractional
            // part that can be losslessly represented with no more than three decimal places,
            // represent as a decimal.
            if self.base > Ratio::from_integer(32) && 1000 % den == 0 {
                let decimal = num * 1000 / den;
                write!(f, "{}", decimal as f32 / 1000.0)?;
            } else {
                write_frac(f, num, den)?;
            }
        } else {
            if self.base != Ratio::from_integer(2) {
                write_frac(f, num, den)?;
            }
            write!(f, "^{}|{}", *self.exp.numer(), *self.exp.denom())?;
        }
        Ok(())
    }
}

impl Factor {
    pub fn new(
        base_numerator: u32,
        base_denominator: u32,
        exp_numerator: i32,
        exp_denominator: i32,
    ) -> anyhow::Result<Self> {
        if base_numerator == 0 || base_denominator == 0 || exp_denominator == 0 {
            bail!("zero may not appear anywhere in base or in exponent denominator");
        }
        Ok(Self {
            base: Ratio::new(base_numerator, base_denominator),
            exp: Ratio::new(exp_numerator, exp_denominator),
        })
    }

    pub fn as_float(&self) -> f32 {
        if self.exp == Ratio::from_integer(1) {
            *self.base.numer() as f32 / *self.base.denom() as f32
        } else {
            let base = *self.base.numer() as f32 / *self.base.denom() as f32;
            let exp = *self.exp.numer() as f32 / *self.exp.denom() as f32;
            base.powf(exp)
        }
    }
}

impl Display for Pitch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for factor in &self.factors {
            if first {
                first = false;
            } else {
                write!(f, "*")?;
            }
            write!(f, "{factor}")?;
        }
        Ok(())
    }
}

impl Pitch {
    pub fn new(factors: Vec<Factor>) -> Self {
        // Canonicalize. This was AI-generated with an extremely detailed spec of the algorithm
        // and subsequently modified.

        // For factors with exponent = 1, we'll multiply them together
        let mut exp_1 = Ratio::<u32>::from_integer(1);

        // For other factors, group by base and sum exponents
        let mut by_base: HashMap<Ratio<u32>, Ratio<i32>> = HashMap::new();

        for factor in factors {
            // Create rationals and reduce to simplest terms
            if factor.exp == Ratio::from_integer(1) {
                // Multiply into our running product
                exp_1 *= factor.base;
            } else {
                // Add exponent to existing base or insert new
                by_base
                    .entry(factor.base)
                    .and_modify(|e| *e += factor.exp)
                    .or_insert(factor.exp);
            }
        }

        let mut result = Vec::new();

        // Add all other factors with base other than 1. If the exponent is negative, adjust
        // the exp_1 base and make it positive.
        for (base, mut exp) in by_base {
            // Normalize to between 0 and denominator
            while *exp.numer() < 0 {
                exp += Ratio::from_integer(1);
                exp_1 /= base;
            }
            while *exp.numer() > *exp.denom() {
                exp -= Ratio::from_integer(1);
                exp_1 *= base;
            }
            if *exp.numer() == 0 {
                continue;
            }
            if base == Ratio::from_integer(1) {
                // The exponent doesn't matter if the base is 1
                exp = Ratio::from_integer(1);
            }
            if exp == Ratio::from_integer(1) {
                exp_1 *= base;
            } else {
                result.push(Factor { base, exp });
            }
        }

        // For consistent results, sort parts with non-1 exponent in decreasing order of exponent.
        // We'll reverse after attaching the exponent-1 factor.
        result.sort_by_key(|f| (f.exp, f.base));

        // Append the exponent-1 factor, taking care to avoid needless multiply by 1
        if result.is_empty() || exp_1 != Ratio::from_integer(1) {
            result.push(Factor {
                base: exp_1,
                exp: Ratio::from_integer(1),
            });
        }
        // Reverse so the exponent-1 factor is first, followed by the other factors in decreasing
        // order of exponent.
        result.reverse();

        Self { factors: result }
    }

    pub fn concat(&self, other: &Self) -> Self {
        Self::new(self.factors.iter().chain(&other.factors).cloned().collect())
    }

    pub fn invert(&self) -> Self {
        let factors = self
            .factors
            .iter()
            .map(|f| {
                let (base_n, base_d, exp_n, exp_d) = if f.exp == Ratio::from_integer(1) {
                    // Exponent is one; take the reciprocal of base
                    (*f.base.denom(), *f.base.numer(), 1, 1)
                } else {
                    // Exponent is not 1; raise to the negation of the exponent
                    (
                        *f.base.numer(),
                        *f.base.denom(),
                        -f.exp.numer(),
                        *f.exp.denom(),
                    )
                };
                Factor::new(base_n, base_d, exp_n, exp_d).unwrap()
            })
            .collect();
        Self::new(factors)
    }

    /// Parse a pitch from a string.
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        parser::pitch().parse(s).map_err(to_anyhow)
    }

    pub fn must_parse(s: &str) -> Self {
        Self::parse(s).unwrap()
    }

    pub fn as_float(&self) -> f32 {
        self.factors
            .iter()
            .fold(1.0f32, |accum, factor| accum * factor.as_float())
    }

    /// Compute a frequency to a midi note number and a pitch bend value using ±2 semitones.
    /// Panics if the frequency is out of range.
    pub fn midi(&self) -> (u8, u16) {
        // TODO: do proper range checking
        let f = self.as_float();
        let n1 = 69.0 + 12.0 * (f / 440.0).log2();
        let note = n1.round() as u8;
        let delta = n1 - note as f32;
        // - pitch bend is 8192 + 8192 * (semitones/bend range)
        // - bend range is typically 2 semitones
        // - 8192*delta/2 is 4096*delta
        // In other words, this the fraction numerator centered at 8192.
        let bend = (8192.0 + (4096.0 * delta).round()) as u16;
        (note, bend)
    }
}

impl FromStr for Pitch {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl std::ops::Mul for &Pitch {
    type Output = Pitch;
    fn mul(self, rhs: Self) -> Self::Output {
        self.concat(rhs)
    }
}

impl std::ops::MulAssign<&Pitch> for Pitch {
    fn mul_assign(&mut self, rhs: &Pitch) {
        let x = self.concat(rhs);
        self.factors = x.factors;
    }
}

impl std::ops::Div<&Pitch> for &Pitch {
    type Output = Pitch;
    fn div(self, rhs: &Pitch) -> Self::Output {
        self.concat(&rhs.invert())
    }
}

impl<'de> Deserialize<'de> for Pitch {
    // This implementation was AI-generated
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PitchVisitor;

        impl<'de> Visitor<'de> for PitchVisitor {
            type Value = Pitch;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a string representing a pitch")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Pitch::parse(v).map_err(E::custom)
            }

            // Accept borrowed Cow<str> as well
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Pitch::parse(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(PitchVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let p = Pitch::parse("440*^3|12*3/2").unwrap();
        assert_eq!(
            p,
            Pitch {
                factors: vec![
                    Factor::new(660, 1, 1, 1).unwrap(),
                    Factor::new(2, 1, 1, 4).unwrap(),
                ],
            }
        );
        let p = Pitch::parse("261.626*3/2").unwrap();
        assert_eq!(
            p,
            Pitch {
                factors: vec![Factor::new(392439, 1000, 1, 1).unwrap(),],
            }
        );
        let p = Pitch::parse("500*4/3^4|7").unwrap();
        assert_eq!(
            p,
            Pitch {
                factors: vec![
                    Factor::new(500, 1, 1, 1).unwrap(),
                    Factor::new(4, 3, 4, 7).unwrap(),
                ],
            }
        );
        let p: Pitch = "3/2".parse().unwrap();
        assert_eq!(
            p,
            Pitch {
                factors: vec![Factor::new(3, 2, 1, 1).unwrap()],
            }
        );
        assert_eq!(p.as_float(), 1.5);
        let p: Pitch = "2*1/2".parse().unwrap();
        assert_eq!(
            p,
            Pitch {
                factors: vec![Factor::new(1, 1, 1, 1).unwrap()],
            }
        );
        let p: Pitch = "4*1/2".parse().unwrap();
        assert_eq!(
            p,
            Pitch {
                factors: vec![Factor::new(2, 1, 1, 1).unwrap()],
            }
        );
        let p: Pitch = "400.1".parse().unwrap();
        assert_eq!(
            p,
            Pitch {
                factors: vec![Factor::new(4001, 10, 1, 1).unwrap()],
            }
        );
        let p: Pitch = "400.10".parse().unwrap();
        assert_eq!(
            p,
            Pitch {
                factors: vec![Factor::new(4001, 10, 1, 1).unwrap()],
            }
        );
        let p: Pitch = "400.123".parse().unwrap();
        assert_eq!(
            p,
            Pitch {
                factors: vec![Factor::new(400123, 1000, 1, 1).unwrap()],
            }
        );
    }

    #[test]
    fn test_parse_errors() {
        assert!(Pitch::parse("").is_err());
        assert!(Pitch::parse("*").is_err());
        assert!(Pitch::parse("2**2").is_err());
        assert!(Pitch::parse("0/4").is_err());
        assert!(Pitch::parse("^3/0").is_err());
        assert!(Pitch::parse("^3|-5").is_err());
        assert!(Pitch::parse("quack").is_err());
        assert!(Pitch::parse("2*").is_err());
        assert!(Pitch::parse("2x").is_err());
        assert!(Pitch::parse("2.").is_err());
        assert!(Pitch::parse(".5").is_err());
        assert!(Pitch::parse("^|12").is_err());
        assert!(Pitch::parse("2^").is_err());
    }

    #[test]
    fn test_equality() -> anyhow::Result<()> {
        // This exercises that pitches are properly canonicalized
        fn check(p1: &str, p2: &str) -> anyhow::Result<()> {
            assert_eq!(Pitch::parse(p1)?, Pitch::parse(p2)?);
            Ok(())
        }

        check("440", "440*3/4*4/3")?;
        check("440", "*440")?;
        check("250*^5|31", "100*^2|31*^3|31*5/2")?;
        check("100*^2|2", "200")?;
        check("660*^-5|12", "330*^7|12")?;
        check("500*^0|31", "500")?;

        let p1 = Pitch::parse("440")?;
        let p2 = &p1 * &Pitch::parse("3/2")?;
        assert_eq!(p2, Pitch::parse("660")?);
        let p3 = &p2 * &Pitch::parse("^-5|12")?;
        assert_eq!(p3, Pitch::parse("330*^7|12")?);

        assert_eq!(p1.to_string(), "440");
        assert_eq!(p2.to_string(), "660");
        assert_eq!(p3.to_string(), "330*^7|12");
        assert_eq!(
            Pitch::parse("3/4*5/3*^1|12*^10|31*3/2^1|2")?.to_string(),
            "5/4*3/2^1|2*^151|372"
        );

        Ok(())
    }

    #[test]
    fn test_as_float() -> anyhow::Result<()> {
        let p = Pitch::parse("440*3/2")?;
        assert_eq!(p.as_float(), 660.0);
        Ok(())
    }

    #[test]
    fn test_invert() {
        let p = Pitch::must_parse("1/2");
        assert_eq!(p.invert().to_string(), "2");
        let p = Pitch::must_parse("^1|2");
        assert_eq!(p.invert().to_string(), "1/2*^1|2");
        assert_eq!(p.invert().invert().to_string(), "^1|2");
    }
}
