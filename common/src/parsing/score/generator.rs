use crate::parsing::diagnostics::{Diagnostics, code};
use crate::parsing::model::{Span, Spanned, Token};
use crate::parsing::pass1;
use crate::parsing::pass1::{Parser1Intermediate, number_intermediate};
use crate::parsing::score::Generator;
use crate::pitch::{Factor, Pitch};
use num_rational::Ratio;
use num_traits::ToPrimitive;
use serde::Serialize;
use std::mem;
use winnow::combinator::{alt, opt, preceded};
use winnow::token::take_while;
use winnow::{LocatingSlice, Parser, combinator};

fn step_factor(divided_interval: Ratio<u32>, divisions: u32, step: i32) -> Factor {
    Factor::new(
        *divided_interval.numer(),
        *divided_interval.denom(),
        step,
        divisions as i32,
    )
    .unwrap()
}

pub(crate) struct NoteGenerator {
    pub divisions: Option<u32>,
    pub divided_interval: Ratio<u32>,
    pub tolerance: Pitch,
}

struct NoteParser<'a> {
    generator: &'a NoteGenerator,
    direction: Option<char>,
    factors: Vec<Factor>,
}

// Leverage of the pass1 parsing machinery to parse generated notes but without using the Pass1
// token types.

#[derive(Serialize, Debug, Clone, Copy)]
enum GenTokenType {
    A {
        is_upper: bool,
        ch: Spanned<char>,
        n: Option<Spanned<u32>>,
    },
    Z {
        is_upper: bool,
        ch: Spanned<char>,
        n: Option<Spanned<u32>>,
    },
    BtoY {
        is_upper: bool,
        harmonic: Spanned<u32>,
    },
    Shift(Spanned<char>),
}
type GenToken<'s> = Spanned<Token<'s, GenTokenType>>;
trait GenParser<'s>: Parser1Intermediate<'s, GenToken<'s>> {}
impl<'s, P: Parser1Intermediate<'s, GenToken<'s>>> GenParser<'s> for P {}
struct Step {
    a: Option<Spanned<u32>>,
    b: Option<Spanned<u32>>,
    c: Option<Spanned<u32>>,
}

fn parse_gen<'s, P, F, T>(p: P, f: F) -> impl GenParser<'s>
where
    P: Parser1Intermediate<'s, T>,
    F: Fn(&'s str, Span, T) -> GenTokenType,
{
    pass1::parse1_intermediate(p, move |raw, span, out| {
        Token::new_spanned(raw, span, f(raw, span, out))
    })
}

fn az_n<'s>(diags: &Diagnostics) -> impl GenParser<'s> {
    parse_gen(
        (
            alt(('a', 'A', 'z', 'Z')).with_span(),
            opt(number_intermediate(diags)),
        ),
        |_raw, _span, ((ch_ch, ch_span), n)| {
            let ch = Spanned::new(ch_span, ch_ch);
            let is_upper = ch_ch.is_ascii_uppercase();
            if ch_ch == 'a' || ch_ch == 'A' {
                GenTokenType::A { is_upper, ch, n }
            } else {
                debug_assert!(ch_ch == 'z' || ch_ch == 'Z');
                GenTokenType::Z { is_upper, ch, n }
            }
        },
    )
}

fn b_to_y<'s>() -> impl GenParser<'s> {
    parse_gen(
        take_while(1, |x: char| {
            ('b'..='y').contains(&x) || ('B'..'Y').contains(&x)
        }),
        |_raw, span, out| {
            let ch = out.chars().next().unwrap();
            let (harmonic, is_upper) = if ch.is_ascii_uppercase() {
                (u32::from(ch) - 0x40, true)
            } else {
                (u32::from(ch) - 0x60, false)
            };
            let harmonic = Spanned::new(span, harmonic);
            GenTokenType::BtoY { is_upper, harmonic }
        },
    )
}

fn shift<'s>() -> impl GenParser<'s> {
    parse_gen(alt(('+', '-', '#', '%')), |_raw, span, out| {
        GenTokenType::Shift(Spanned::new(span, out))
    })
}

fn step<'s>(diags: &Diagnostics) -> impl Parser1Intermediate<'s, Step> {
    pass1::parse1_intermediate(
        preceded(
            '!',
            opt((
                number_intermediate(diags),
                opt(preceded(
                    '/',
                    (
                        number_intermediate(diags),
                        opt(preceded('/', number_intermediate(diags))),
                    ),
                )),
            )),
        ),
        |_raw, _span, maybe_abc| {
            let (a, b, c) = match maybe_abc {
                None => (None, None, None),
                Some((a, maybe_bc)) => {
                    let (b, c) = match maybe_bc {
                        None => (None, None),
                        Some((b, maybe_c)) => (Some(b), maybe_c),
                    };
                    (Some(a), b, c)
                }
            };
            Step { a, b, c }
        },
    )
}

impl<'a> NoteParser<'a> {
    fn handle_harmonic(&mut self, harmonic: u32, up: bool) {
        debug_assert!(harmonic >= 2);
        let (num, den) = if up {
            (harmonic, harmonic - 1)
        } else {
            (harmonic - 1, harmonic)
        };
        self.factors.push(Factor::new(num, den, 1, 1).unwrap());
    }

    fn parse(mut self, diags: &Diagnostics, name: &Spanned<&str>) -> Option<Pitch> {
        let input = LocatingSlice::new(name.value);
        let Result::<(Vec<GenToken>, Option<Step>), _>::Ok((parts, step)) = (
            combinator::repeat(1.., alt((az_n(diags), b_to_y(), shift()))),
            opt(step(diags)),
        )
            .parse(input)
        else {
            diags.err(
                code::GENERATED_NOTE,
                0..name.value.len(),
                "unable to parse as generated note",
            );
            return None;
        };
        let (divided_interval, divisions) = match step {
            None => (self.generator.divided_interval, self.generator.divisions),
            Some(Step { a, b, c }) => {
                // If there are no values, we don't overlay. If there is one value, it is the number
                // of divisions of the specified division interval. If two values, the first is a
                // division interval, and the second is divisions. If three values, the first two
                // are a rational division interval, and the third is divisions. The parser
                // guarantees that `b` is Some if `c` is Some and `a` is Some if `b` is Some.
                match c {
                    None => match b {
                        None => match a {
                            None => (self.generator.divided_interval, None),
                            Some(a) => {
                                // If there is only one value, it is the number of divisions of the
                                // default division interval.
                                (self.generator.divided_interval, Some(a.value))
                            }
                        },
                        Some(b) => {
                            // If there are two values, the first is a division interval integer and
                            // the second is the number of divisions
                            (Ratio::from_integer(a.unwrap().value), Some(b.value))
                        }
                    },
                    Some(c) => {
                        // If there are three values, the first two are a division interval and the
                        // third is the number of divisions
                        (
                            Ratio::new(a.unwrap().value, b.unwrap().value),
                            Some(c.value),
                        )
                    }
                }
            }
        };
        for part in parts {
            match part.value.t {
                GenTokenType::A { is_upper, ch: _, n } => {
                    match divisions {
                        None => {
                            if let Some(n) = n
                                && n.value > 0
                            {
                                diags.err(
                                    code::GENERATED_NOTE,
                                    n.span,
                                    "for pure Just Intonation step must be 0 or omitted",
                                );
                            }
                        }
                        Some(divisions) => {
                            if let Some(n) = n
                                && n.value >= divisions
                            {
                                diags.err(
                                    code::GENERATED_NOTE,
                                    n.span,
                                    format!(
                                        "step number must be ≤ {} (divisions - 1)",
                                        divisions - 1
                                    ),
                                );
                            } else {
                                // This is an EDO step. A value of zero doesn't change the pitch, so
                                // omit.
                                let n_value = n.map(Spanned::value).unwrap_or(0) as i32;
                                if n_value > 0 {
                                    let steps = if is_upper { n_value } else { -n_value };
                                    self.factors.push(step_factor(
                                        divided_interval,
                                        divisions,
                                        steps,
                                    ));
                                }
                            }
                        }
                    }
                }
                GenTokenType::Z { is_upper, ch, n } => {
                    match n {
                        None => {
                            // We could make this mandatory in the combinator, but make it optional
                            // and reporting an error makes it possible to issue a better error for
                            // z without a number.
                            diags.err(
                                code::GENERATED_NOTE,
                                ch.span,
                                format!("{} must be followed by a number >= 2", ch.value),
                            )
                        }
                        Some(n) => {
                            if n.value >= 2 {
                                self.handle_harmonic(n.value, is_upper);
                            } else {
                                diags.err(
                                    code::GENERATED_NOTE,
                                    n.span,
                                    format!("number after {} must be >= 2", ch.value),
                                );
                            }
                        }
                    }
                }
                GenTokenType::BtoY { is_upper, harmonic } => {
                    self.handle_harmonic(harmonic.value, is_upper);
                }
                GenTokenType::Shift(ch) => {
                    if ch.value == '+' || ch.value == '-' {
                        if let Some(divisions) = divisions {
                            let step = if ch.value == '+' { 1 } else { -1 };
                            self.factors
                                .push(step_factor(divided_interval, divisions, step))
                        } else {
                            diags.err(
                                code::GENERATED_NOTE,
                                ch.span,
                                "+ and - are not permitted in pure Just Intonation generated scales",
                            );
                        }
                    } else {
                        debug_assert!(ch.value == '#' || ch.value == '%');
                        if self.direction.is_some() {
                            diags.err(
                                code::GENERATED_NOTE,
                                ch.span,
                                "# or % may appear at most once",
                            );
                        } else {
                            self.direction = Some(ch.value);
                        }
                    }
                }
            }
        }
        if diags.has_errors() {
            return None;
        }

        let pitch = if self.factors.is_empty() {
            Pitch::unit()
        } else {
            if let Some(divisions) = divisions {
                // Find the closest scale degree.
                let factors = mem::take(&mut self.factors);
                let divided_interval_f32 = divided_interval.to_f32().unwrap();
                let step =
                    Pitch::new(factors).as_float().log(divided_interval_f32) * divisions as f32;
                let tolerance_steps = self
                    .generator
                    .tolerance
                    .as_float()
                    .log(divided_interval_f32)
                    * divisions as f32;
                let rounded = step.round();
                let degree = if let Some(d) = self.direction
                    && (rounded - step).abs() > tolerance_steps
                {
                    if d == '#' {
                        step.ceil() as i32
                    } else {
                        debug_assert!(d == '%');
                        step.floor() as i32
                    }
                } else {
                    rounded as i32
                };
                self.factors = vec![step_factor(divided_interval, divisions, degree)];
            }
            Pitch::new(self.factors)
        };
        Some(pitch)
    }
}

impl Generator for NoteGenerator {
    fn get_note(&self, diags: &Diagnostics, name: &Spanned<&str>) -> Option<Pitch> {
        let parser = NoteParser {
            generator: self,
            direction: None,
            factors: Vec::new(),
        };
        let temp_diags = Diagnostics::new();
        let r = parser.parse(&temp_diags, name);
        diags.merge_with_offset(temp_diags, name.span.start);
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_note(n: &str) -> Spanned<&str> {
        Spanned::new(0..n.len(), n)
    }

    #[test]
    fn test_generator() {
        // TODO: test when divided_interval != cycle

        // Error conditions are tested through parser tests.
        let g = NoteGenerator {
            divisions: None,
            divided_interval: Ratio::from_integer(2),
            tolerance: Pitch::unit(),
        };
        // Pure JI (Just Intonation)
        let diags = Diagnostics::new();
        for (name, wanted) in [
            ("a", "1"),
            ("a0", "1"),
            ("A", "1"),
            ("A0", "1"),
            ("B", "2"),
            ("q", "16/17"),
            ("Beee", "128/125"),
            ("Cl", "11/8"),
            ("A#", "1"),
            ("A%", "1"),
            ("Z30", "30/29"),
            ("z30", "29/30"),
            ("C!12", "^7|12"),
            ("C!3/18", "3^7|18"),
            ("C!9/4/8", "9/4^1|2"),
            ("C!4/3/5", "4/3^7|5"),
            ("C!3/2/5", "3/2"),
            ("E+!17", "^6|17"),
        ] {
            let pitch = g.get_note(&diags, &make_note(name));
            assert!(pitch.is_some(), "failed to parse {name}");
            assert_eq!(pitch.unwrap(), Pitch::must_parse(wanted));
        }
        // EDO Overlay
        fn make_g(divided_interval: Ratio<u32>, divisions: u32, tolerance: Pitch) -> NoteGenerator {
            NoteGenerator {
                divisions: Some(divisions),
                divided_interval,
                tolerance,
            }
        }
        // Zero tolerance
        let pitch_of = |g: &NoteGenerator, n: &str| g.get_note(&diags, &make_note(n)).unwrap();
        let g = make_g(Ratio::from_integer(2), 17, Pitch::unit());
        assert_eq!(pitch_of(&g, "D"), Pitch::must_parse("^7|17"));
        assert_eq!(pitch_of(&g, "D%"), Pitch::must_parse("^7|17"));
        assert_eq!(pitch_of(&g, "D#"), Pitch::must_parse("^8|17"));
        // 4/3 is less than 4¢ from ^7|17.
        let g = make_g(Ratio::from_integer(2), 17, Pitch::must_parse("^1|400"));
        assert_eq!(pitch_of(&g, "D%"), Pitch::must_parse("^7|17"));
        assert_eq!(pitch_of(&g, "D#"), Pitch::must_parse("^8|17"));
        let g = make_g(Ratio::from_integer(2), 17, Pitch::must_parse("^1|300"));
        assert_eq!(pitch_of(&g, "D%"), Pitch::must_parse("^7|17"));
        assert_eq!(pitch_of(&g, "D#"), Pitch::must_parse("^7|17"));
        assert_eq!(pitch_of(&g, "E"), Pitch::must_parse("^5|17"));
        assert_eq!(pitch_of(&g, "E#"), Pitch::must_parse("^6|17"));
        assert_eq!(pitch_of(&g, "JK"), Pitch::must_parse("^5|17")); // 11/9
        let g = make_g(Ratio::from_integer(2), 31, Pitch::must_parse("^1|240")); // 5¢
        assert_eq!(pitch_of(&g, "Bh"), Pitch::must_parse("^25|31")); // 7/4
        assert_eq!(pitch_of(&g, "Bh#"), Pitch::must_parse("^25|31")); // 7/4
        assert_eq!(pitch_of(&g, "Bh%"), Pitch::must_parse("^25|31")); // 7/4
        assert_eq!(pitch_of(&g, "A3Bh%"), Pitch::must_parse("^28|31"));
        assert_eq!(pitch_of(&g, "a3Bh%"), Pitch::must_parse("^22|31"));
        assert_eq!(pitch_of(&g, "a3Bh%+"), Pitch::must_parse("^23|31"));
        assert_eq!(pitch_of(&g, "a3Bh%+Beee"), Pitch::must_parse("^24|31"));
        let g = make_g(Ratio::from_integer(2), 41, Pitch::must_parse("^1|240")); // 5¢
        assert_eq!(pitch_of(&g, "E"), Pitch::must_parse("^13|41"));
        // Division interval size other than 2. Neutral third is ~2 steps in 4 divisions of a perfect fifth.
        let g = make_g(Ratio::new(3, 2), 4, Pitch::unit());
        assert_eq!(pitch_of(&g, "JI"), Pitch::must_parse("3/2^1|2"));
        assert_eq!(pitch_of(&g, "JI+"), Pitch::must_parse("3/2^3|4"));
        assert_eq!(pitch_of(&g, "JI-"), Pitch::must_parse("3/2^1|4"));
        // Override specified overlay
        assert_eq!(pitch_of(&g, "D"), Pitch::must_parse("3/2^3|4"));
        assert_eq!(pitch_of(&g, "D!"), Pitch::must_parse("4/3"));
        assert_eq!(pitch_of(&g, "D!2/12"), Pitch::must_parse("^5|12"));
        assert_eq!(pitch_of(&g, "D!3/18"), Pitch::must_parse("3^5|18"));
        assert_eq!(pitch_of(&g, "D!7"), Pitch::must_parse("3/2^5|7"));

        assert!(!diags.has_errors());
    }
}
