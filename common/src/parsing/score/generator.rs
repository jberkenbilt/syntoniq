use crate::parsing::diagnostics::{Diagnostics, code};
use crate::parsing::model::{Span, Spanned};
use crate::parsing::score::Generator;
use crate::pitch::{Factor, Pitch};
use num_rational::Ratio;
use num_traits::ToPrimitive;
use std::mem;

pub(crate) struct Overlay {
    pub cycle: Ratio<u32>,
    pub divisions: u32,
    pub tolerance: Pitch,
}
impl Overlay {
    fn factor(&self, step: i32) -> Factor {
        Factor::new(
            *self.cycle.numer(),
            *self.cycle.denom(),
            step,
            self.divisions as i32,
        )
        .unwrap()
    }
}

pub(crate) struct NoteGenerator {
    pub overlay: Option<Overlay>,
}

struct NoteParser<'a> {
    diags: &'a Diagnostics,
    generator: &'a NoteGenerator,
    num_arg: u32,
    num_span: Span,
    allow_num: bool,
    pending_num_char: Option<char>,
    direction: Option<char>,
    num_errors: usize,
    factors: Vec<Factor>,
}

impl<'a> NoteParser<'a> {
    fn handle_num(&mut self) {
        let Some(letter) = self.pending_num_char.take() else {
            return;
        };
        let span = self.num_span;
        self.num_span = Span::from(0..1);
        let num_arg = self.num_arg;
        self.num_arg = 0;
        if letter == 'a' || letter == 'A' {
            match &self.generator.overlay {
                None => {
                    if num_arg > 0 {
                        self.diags.err(
                            code::GENERATED_NOTE,
                            span,
                            "for pure Just Intonation step must be 0 or omitted",
                        );
                    }
                }
                Some(overlay) => {
                    if num_arg < overlay.divisions {
                        // This is an EDO step. A value of zero doesn't change the pitch, so
                        // omit.
                        if num_arg > 0 {
                            let steps = if letter == 'A' {
                                num_arg as i32
                            } else {
                                -(num_arg as i32)
                            };
                            self.factors.push(overlay.factor(steps));
                        }
                    } else {
                        self.diags.err(
                            code::GENERATED_NOTE,
                            span,
                            format!(
                                "the maximum allowed step for this scale is {}",
                                overlay.divisions - 1
                            ),
                        );
                    }
                }
            }
        } else {
            // This only gets called when there is a pending letter,
            debug_assert!(letter == 'z' || letter == 'Z');
            if num_arg >= 2 {
                self.handle_harmonic(num_arg, letter.is_uppercase());
            } else {
                self.diags
                    .err(code::GENERATED_NOTE, span, "argument to Z must be >= 2");
            }
        }
    }

    fn handle_harmonic(&mut self, harmonic: u32, up: bool) {
        debug_assert!(harmonic >= 2);
        let (num, den) = if up {
            (harmonic, harmonic - 1)
        } else {
            (harmonic - 1, harmonic)
        };
        self.factors.push(Factor::new(num, den, 1, 1).unwrap());
    }

    fn parse(mut self, name: &Spanned<&str>) -> Option<Pitch> {
        let mut pos = name.span.start;
        for ch in name.value.chars() {
            let span = Span::from(pos..pos + 1);
            if ch.is_ascii_digit() {
                if self.num_span.start == 0 {
                    self.num_span = span;
                } else {
                    self.num_span.end += 1;
                }
                if self.allow_num {
                    if let Some(x) = self
                        .num_arg
                        .checked_mul(10)
                        .and_then(|x| x.checked_add(ch.to_digit(10).unwrap()))
                    {
                        self.num_arg = x;
                    } else {
                        self.diags.err(
                            code::GENERATED_NOTE,
                            span,
                            "overflow handling numeric value",
                        );
                    }
                } else {
                    self.diags
                        .err(code::GENERATED_NOTE, span, "a number is not permitted here");
                }
            } else {
                self.allow_num = false;
                self.handle_num();
                if ch.is_ascii_alphabetic() {
                    if ch == 'a' || ch == 'A' || ch == 'z' || ch == 'Z' {
                        self.allow_num = true;
                        self.pending_num_char = Some(ch)
                    } else {
                        // Get the ordinal position of the letter. Upper-case letters go up, and
                        // lower-case letters go down.
                        let (harmonic, up) = if ch.is_ascii_uppercase() {
                            (u32::from(ch) - 0x40, true)
                        } else {
                            (u32::from(ch) - 0x60, false)
                        };
                        self.handle_harmonic(harmonic, up);
                    }
                } else if ch == '+' || ch == '-' {
                    if let Some(overlay) = &self.generator.overlay {
                        let step = if ch == '+' { 1 } else { -1 };
                        self.factors.push(overlay.factor(step))
                    } else {
                        self.diags.err(
                            code::GENERATED_NOTE,
                            span,
                            "+ and - are not permitted in pure Just Intonation generated scales",
                        );
                    }
                } else if ch == '#' || ch == '%' {
                    if self.direction.is_some() {
                        self.diags.err(
                            code::GENERATED_NOTE,
                            span,
                            "# or % may appear at most once",
                        );
                    } else {
                        self.direction = Some(ch);
                    }
                } else {
                    self.diags.err(
                        code::GENERATED_NOTE,
                        span,
                        "this character is not permitted here",
                    );
                }
            }
            pos += ch.len_utf8();
        }
        self.handle_num();
        if self.diags.num_errors() > self.num_errors {
            return None;
        }
        let pitch = if self.factors.is_empty() {
            Pitch::unit()
        } else {
            if let Some(overlay) = &self.generator.overlay {
                // Find the closest scale degree.
                let factors = mem::take(&mut self.factors);
                let cycle_f32 = overlay.cycle.to_f32().unwrap();
                let step = Pitch::new(factors).as_float().log(cycle_f32) * overlay.divisions as f32;
                let tolerance_steps =
                    overlay.tolerance.as_float().log(cycle_f32) * overlay.divisions as f32;
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
                self.factors = vec![overlay.factor(degree)];
            }
            Pitch::new(self.factors)
        };
        Some(pitch)
    }
}

impl Generator for NoteGenerator {
    fn get_note(&self, diags: &Diagnostics, name: &Spanned<&str>) -> Option<Pitch> {
        let parser = NoteParser {
            diags,
            generator: self,
            num_arg: 0,
            num_span: Span::from(0..1),
            allow_num: false,
            pending_num_char: None,
            direction: None,
            num_errors: diags.num_errors(),
            factors: Vec::new(),
        };
        parser.parse(name)
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
        // Error conditions are tested through parser tests.
        let g = NoteGenerator { overlay: None };
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
        ] {
            let pitch = g.get_note(&diags, &make_note(name));
            assert!(pitch.is_some());
            assert_eq!(pitch.unwrap(), Pitch::must_parse(wanted));
        }
        // EDO Overlay
        fn make_g(cycle: Ratio<u32>, divisions: u32, tolerance: Pitch) -> NoteGenerator {
            NoteGenerator {
                overlay: Some(Overlay {
                    cycle,
                    divisions,
                    tolerance,
                }),
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
        // Cycle size other than 2. Neutral third is ~2 steps in 4 divisions of a perfect fifth.
        let g = make_g(Ratio::new(3, 2), 4, Pitch::unit());
        assert_eq!(pitch_of(&g, "JI"), Pitch::must_parse("3/2^1|2"));
        assert_eq!(pitch_of(&g, "JI+"), Pitch::must_parse("3/2^3|4"));
        assert_eq!(pitch_of(&g, "JI-"), Pitch::must_parse("3/2^1|4"));

        assert!(!diags.has_errors());
    }
}
