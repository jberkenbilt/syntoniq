use anyhow::{anyhow, bail};
use regex::Regex;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, de};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Pitch {
    pub base: f32,
    pub multipliers: Vec<Multiplier>,
}

#[derive(Debug, PartialEq)]
pub enum Multiplier {
    Ratio(Ratio),
    Exponent(Exponent),
}
#[derive(Debug, PartialEq)]
pub struct Ratio {
    pub numerator: u8,
    pub denominator: u8,
}
#[derive(Debug, PartialEq)]
pub struct Exponent {
    pub exp_numerator: u8,
    pub exp_denominator: u8,
    pub base_numerator: u8,
    pub base_denominator: u8,
}

impl Multiplier {
    pub fn as_float(&self) -> f32 {
        match self {
            Multiplier::Ratio(r) => r.numerator as f32 / r.denominator as f32,
            Multiplier::Exponent(ed) => {
                let base = ed.base_numerator as f32 / ed.base_denominator as f32;
                let exp = ed.exp_numerator as f32 / ed.exp_denominator as f32;
                base.powf(exp)
            }
        }
    }
}

impl Pitch {
    /// Parse a pitch from a string.
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        // This function and its tests were AI-generated
        // Main regex for a pitch
        let re = Regex::new(
            r"(?x)
                ^
                (?P<base> [0-9]*\.?[0-9]* ) # may be empty
                (?P<multipliers>
                    (?:\* [^*]+)*
                )
                $
            ",
        )?;
        let caps = re
            .captures(s.trim())
            .ok_or_else(|| anyhow!("invalid syntax for pitch"))?;

        // Parse base
        let base = caps["base"].parse().ok().unwrap_or(1.0);

        // Parse multipliers (zero or more)
        let multipliers_str = &caps["multipliers"];
        let mut multipliers = Vec::new();

        if !multipliers_str.trim().is_empty() {
            // Regex for individual multiplier (either ratio or equal division)
            let re_mult = Regex::new(
                r"(?x)
                    \*
                    (?:
                        # Ratio: a/b
                        (?P<ratio>
                            (?P<r_num>\d+)
                            /
                            (?P<r_den>\d+)
                        )
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
                        )
                    )
                ",
            )?;

            for cap in re_mult.captures_iter(multipliers_str) {
                // Ratio, eg. 3/2
                if cap.name("ratio").is_some() {
                    let num: u8 = cap["r_num"].parse()?;
                    let den: u8 = cap["r_den"].parse()?;
                    multipliers.push(Multiplier::Ratio(Ratio {
                        numerator: num,
                        denominator: den,
                    }));
                    continue;
                }
                // Exponent, eg. 3\12, 4\7/4/3
                if cap.name("exp").is_some() {
                    let exp_numerator: u8 = cap["e_num"].parse()?;
                    let exp_denominator: u8 = cap["e_den"].parse()?;

                    let base_numerator: u8 = if let Some(num) = cap.name("e_base_num") {
                        num.as_str().parse()?
                    } else {
                        2 // default base numerator
                    };
                    let base_denominator: u8 = if let Some(den) = cap.name("e_base_den") {
                        den.as_str().parse()?
                    } else {
                        1 // default base denominator
                    };

                    multipliers.push(Multiplier::Exponent(Exponent {
                        exp_numerator,
                        exp_denominator,
                        base_numerator,
                        base_denominator,
                    }));
                    continue;
                }
                bail!("unparsed multiplier: {}", &cap[0]);
            }
        }

        Ok(Self { base, multipliers })
    }

    pub fn as_float(&self) -> f32 {
        let mut result = self.base;
        for m in &self.multipliers {
            result *= m.as_float();
        }
        result
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
                base: 440.0,
                multipliers: vec![
                    Multiplier::Exponent(Exponent {
                        exp_numerator: 3,
                        exp_denominator: 12,
                        base_numerator: 2,
                        base_denominator: 1,
                    }),
                    Multiplier::Ratio(Ratio {
                        numerator: 3,
                        denominator: 2,
                    }),
                ],
            }
        );
        let p = Pitch::parse("261.626*3/2").unwrap();
        assert_eq!(
            p,
            Pitch {
                base: 261.626,
                multipliers: vec![Multiplier::Ratio(Ratio {
                    numerator: 3,
                    denominator: 2,
                }),],
            }
        );
        let p = Pitch::parse("500*4\\7/4/3").unwrap();
        assert_eq!(
            p,
            Pitch {
                base: 500.0,
                multipliers: vec![Multiplier::Exponent(Exponent {
                    exp_numerator: 4,
                    exp_denominator: 7,
                    base_numerator: 4,
                    base_denominator: 3,
                }),],
            }
        );
        let p: Pitch = "*3/2".parse().unwrap();
        assert_eq!(
            p,
            Pitch {
                base: 1.0,
                multipliers: vec![Multiplier::Ratio(Ratio {
                    numerator: 3,
                    denominator: 2
                })],
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
