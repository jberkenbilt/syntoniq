use anyhow::bail;
use regex::Regex;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, de};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Pitch {
    factors: Vec<Factor>,
}

#[derive(Debug, PartialEq)]
pub struct Factor {
    base_numerator: u32,
    base_denominator: u32,
    exp_numerator: u32,
    exp_denominator: u32,
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
            base_numerator,
            base_denominator,
            exp_numerator,
            exp_denominator,
        })
    }
    pub fn as_float(&self) -> f32 {
        if self.exp_numerator == self.exp_denominator {
            self.base_numerator as f32 / self.base_denominator as f32
        } else {
            let base = self.base_numerator as f32 / self.base_denominator as f32;
            let exp = self.exp_numerator as f32 / self.exp_denominator as f32;
            base.powf(exp)
        }
    }
}

impl Pitch {
    pub fn new(factors: Vec<Factor>) -> Self {
        Self { factors }
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
                    let (num_frac, frac_mult) = match cap.name("r_num_frac") {
                        None => (0, 1),
                        Some(x) => (x.as_str().parse()?, 1000),
                    };
                    let num: u32 = cap["r_num"].parse::<u32>()? * frac_mult + num_frac;
                    let den: u32 = match cap.name("r_den") {
                        None => 1,
                        Some(x) => x.as_str().parse()?,
                    } * frac_mult;
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

        Ok(Self { factors })
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
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PitchVisitor;

        impl<'de> Visitor<'de> for PitchVisitor {
            type Value = Pitch;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a pitch")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Pitch::parse(v).map_err(E::custom)
            }

            // Accept borrowed Cow<str> as well
            fn visit_borrowed_str<E>(self, v: &'de str) -> std::result::Result<Self::Value, E>
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
                    Factor::new(440, 1, 1, 1).unwrap(),
                    Factor::new(2, 1, 3, 12).unwrap(),
                    Factor::new(3, 2, 1, 1).unwrap(),
                ],
            }
        );
        let p = Pitch::parse("261.626*3/2").unwrap();
        assert_eq!(
            p,
            Pitch {
                factors: vec![
                    Factor::new(261626, 1000, 1, 1).unwrap(),
                    Factor::new(3, 2, 1, 1).unwrap(),
                ],
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
    fn test_as_float() -> anyhow::Result<()> {
        let p = Pitch::parse("440*3/2")?;
        assert_eq!(p.as_float(), 660.0);
        Ok(())
    }
}
