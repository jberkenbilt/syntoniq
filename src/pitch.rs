use anyhow::bail;
use num_rational::Ratio;
use regex::Regex;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, de};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pitch {
    factors: Vec<Factor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Factor {
    base: Ratio<u32>,
    exp: Ratio<u32>,
}

impl Factor {
    pub fn new(
        base_numerator: u32,
        base_denominator: u32,
        exp_numerator: u32,
        exp_denominator: u32,
    ) -> anyhow::Result<Self> {
        if base_numerator == 0
            || base_denominator == 0
            || exp_numerator == 0
            || exp_denominator == 0
        {
            bail!("zero may not appear in pitch specification");
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

impl Pitch {
    pub fn new(factors: Vec<Factor>) -> Self {
        // Canonicalize. This was AI-generated with an extremely detailed spec of the algorithm.

        // For factors with exponent = 1, we'll multiply them together
        let mut exp_1 = Ratio::<u32>::from_integer(1);

        // For other factors, group by base and sum exponents
        let mut by_base: HashMap<Ratio<u32>, Ratio<u32>> = HashMap::new();

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

        // Build result vector
        let mut result = Vec::new();

        // Add the multiplied factors if we had any
        if exp_1 != Ratio::from_integer(1) {
            result.push(Factor {
                base: exp_1,
                exp: Ratio::from_integer(1),
            });
        }

        // Add all the other factors
        for (base, mut exp) in by_base {
            // Skip if exponent reduces to 0 (shouldn't happen with your validation)
            if *exp.numer() != 0 {
                if base == Ratio::from_integer(1) {
                    // The exponent doesn't matter if the base is 1
                    exp = Ratio::from_integer(1);
                }
                result.push(Factor { base, exp });
            }
        }

        // Sort by the tuple as specified
        result.sort_by_key(|f| (f.base, f.exp));
        result.reverse();

        Self { factors: result }
    }

    pub fn concat(&self, mut other: Self) -> Self {
        other.factors.extend(self.factors.iter().cloned());
        Self::new(other.factors)
    }

    /// Parse a pitch from a string.
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        // This function and its tests were originally AI-generated, then substantially modified.
        let multiplier_re = Regex::new(
            r"(?x)
                    (?:
                        # Ratio: a/b
                        (?P<ratio>
                            (?P<r_num>\d+)(?:\.(?P<r_num_frac>\d{1,3}))?
                            (?:/(?P<r_den>\d+))?
                        $)
                        |
                        # Exponent: a\b[c[/d]]
                        (?P<exp>
                            (?P<e_num>\d+)
                            \\
                            (?P<e_den>\d+)
                            (?:
                                /
                                (?P<e_base_num>\d+)
                                (?:
                                    /
                                    (?P<e_base_den>\d+)
                                )?
                            )?
                        $)
                    )
                ",
        )?;

        let factors_str = s.split('*');
        let mut factors = Vec::new();
        for factor in factors_str {
            // Regex for individual multiplier (either ratio or equal division)
            for cap in multiplier_re.captures_iter(factor) {
                // Ratio, eg. 3/2
                if cap.name("ratio").is_some() {
                    let (num_frac, scale) = match cap.name("r_num_frac") {
                        None => (0, 1),
                        Some(x) => (x.as_str().parse()?, 1000),
                    };
                    let num: u32 = cap["r_num"].parse::<u32>()? * scale + num_frac;
                    let den: u32 = match cap.name("r_den") {
                        None => 1,
                        Some(x) => x.as_str().parse()?,
                    } * scale;
                    factors.push(Factor::new(num, den, 1, 1)?);
                    continue;
                }
                // Exponent, eg. 3\12, 4\7/4/3
                if cap.name("exp").is_some() {
                    let exp_numerator: u32 = cap["e_num"].parse()?;
                    let exp_denominator: u32 = cap["e_den"].parse()?;

                    let base_numerator: u32 = if let Some(num) = cap.name("e_base_num") {
                        num.as_str().parse()?
                    } else {
                        2 // default base numerator
                    };
                    let base_denominator: u32 = if let Some(den) = cap.name("e_base_den") {
                        den.as_str().parse()?
                    } else {
                        1 // default base denominator
                    };

                    factors.push(Factor::new(
                        base_numerator,
                        base_denominator,
                        exp_numerator,
                        exp_denominator,
                    )?);
                    continue;
                }
                bail!("unparsed multiplier: {}", &cap[0]);
            }
        }
        if factors.is_empty() {
            bail!("pitch may not be empty");
        }

        Ok(Self::new(factors))
    }

    pub fn as_float(&self) -> f32 {
        self.factors
            .iter()
            .fold(1.0f32, |accum, factor| accum * factor.as_float())
    }
}

impl FromStr for Pitch {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
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

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
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
        let p = Pitch::parse("440*3\\12*3/2").unwrap();
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
        let p = Pitch::parse("500*4\\7/4/3").unwrap();
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
    }

    #[test]
    fn test_equality() -> anyhow::Result<()> {
        // This exercises that pitches are properly canonicalized
        fn check(p1: &str, p2: &str) -> anyhow::Result<()> {
            assert_eq!(Pitch::parse(p1)?, Pitch::parse(p2)?);
            Ok(())
        }

        check("440", "440*3/4*4/3")?;
        check("250*5\\31", "100*2\\31*3\\31*5/2")?;
        check("100*2\\2", "200")?;

        let p1 = Pitch::parse("440")?;
        let p2 = p1.concat(Pitch::parse("3/2")?);
        assert_eq!(p2, Pitch::parse("660")?);

        Ok(())
    }

    #[test]
    fn test_as_float() -> anyhow::Result<()> {
        let p = Pitch::parse("440*3/2")?;
        assert_eq!(p.as_float(), 660.0);
        Ok(())
    }
}
