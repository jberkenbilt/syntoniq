use crate::parsing::diagnostics::{Diagnostics, code};
use crate::parsing::model::{Span, Spanned, Token};
use crate::parsing::pass1;
use crate::parsing::pass1::{Parser1Intermediate, number_intermediate};
use crate::parsing::score::{Assignments, Generator};
use crate::pitch::{Factor, Pitch};
use num_rational::Ratio;
use num_traits::ToPrimitive;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
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

/// Candidate represents a note's closeness to a step.
struct Candidate {
    closest_step: i32,
    delta: f64,
    note_path: NotePath,
}
impl Candidate {
    /// Call as new_candidate.closer(old_candidate).
    fn closer_than(&self, other: &Candidate) -> bool {
        // If two pitches are the same distance from their closest step, consider the other one
        // closer. This way, earlier, "simpler" notes take precedence when this code is called the
        // way the note assignment code calls it.
        self.delta.abs() < other.delta.abs()
    }

    fn step_letter(step: i32) -> char {
        // Step n represents n/n-1, which is the uppercase letter in position n, e.g. 3 -> 3/2 -> C.
        // Step -n represents the reciprocal, e.g. -3 -> 2/3 -> c.
        if step > 0 {
            char::from_u32((64 + step) as u32).unwrap()
        } else {
            char::from_u32((96 - step) as u32).unwrap()
        }
    }

    fn name(&self) -> Cow<'static, str> {
        // Prepend `B` (2/1) for each octave, then append the letter for each step.
        let mut s = String::new();
        for _ in 0..self.note_path.octaves {
            s.push('B');
        }
        s.push(Self::step_letter(self.note_path.step1));
        if self.note_path.step2 != 0 {
            s.push(Self::step_letter(self.note_path.step2));
        }
        // Since we only consider candidates that are closest to a step, it is never necessary to
        // add `%` or `#` to the note name. This serves as a visual indicator that the match is
        // slightly farther away from the note. It seldom appears because we can get quite close
        // with two steps to most pitches.
        if self.delta > 0.2 {
            s.push('%');
        } else if self.delta < -0.2 {
            s.push('#');
        }
        Cow::Owned(s)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
/// When assigning note names, we limit ourselves to notes with at most two intervals other than
/// `B` (ratio 2). This is an efficient intermediate representation. Logic is divided between
/// NotePath and Candidate.
struct NotePath {
    octaves: i32,
    step1: i32,
    step2: i32,
}
impl NotePath {
    fn candidate(
        octaves: i32,
        step1: i32,
        step2: i32,
        divided_interval: f64,
        divisions: i32,
    ) -> Option<Candidate> {
        let note_path = NotePath {
            octaves,
            step1,
            step2,
        };
        note_path.into_candidate(divided_interval, divisions)
    }

    fn step_to_ratio(step: i32) -> Ratio<i32> {
        if step > 0 {
            // 5 -> 5/4
            Ratio::new(step, step - 1)
        } else if step < 0 {
            // -5 -> 4/5
            Ratio::new(-step - 1, -step)
        } else {
            Ratio::from_integer(1)
        }
    }

    fn into_candidate(self, divided_interval: f64, divisions: i32) -> Option<Candidate> {
        // Find which step the ratio is closest to.
        let step1 = Self::step_to_ratio(self.step1);
        let step2 = Self::step_to_ratio(self.step2);
        let val = (step1 * step2).to_f64().unwrap() * 2i32.pow(self.octaves as u32) as f64;
        if !(1.0..divided_interval).contains(&val) {
            // If this falls outside the interval, discard it regardless of how close it is.
            return None;
        }
        let steps = val.log(divided_interval) * divisions as f64;
        let closest_step = steps.round() as i32;
        let delta = steps - steps.round();
        if delta.abs() > 0.4 {
            // We could use >= 0.5 here, but we require the candidate to be slightly closer to the
            // target to create less auditory ambiguity.
            return None;
        }
        Some(Candidate {
            closest_step,
            delta,
            note_path: self,
        })
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

    fn assign_generated_notes(&self) -> Assignments {
        // This code tries to assign the "simplest" and "closest" generated note to each degree of a
        // divided scale. In practice, it will not always pick what a user would pick; e.g., in
        // 17-EDO, this picks `Dx` for the sixth scale degree, while a user would probably call it
        // `E%`, but it produces something that tells a reader who is used to the generated note
        // system something about where the pitch is. The main place someone would see these would
        // be in an isomorphic keyboard layout based on a generated scale, but they can also be
        // hints. An experienced user might see the sequence `GU`, `Dx` and recognize that `E` (5/4)
        // is going to fall somewhere between `GU` and `Dx`. `GU` is a little above 7/6, and `Dx` is
        // a little below 4/3. Other cues, such as color and auditory feedback will contribute. See
        // comments in line.
        let Some(divisions) = self.divisions.map(|x| x as i32) else {
            // If this is pure JI, we have infinite scale degrees, so we just assign notes that the
            // user actually uses.
            return Default::default();
        };
        // Generate two lists of candidates in priority order. `candidates1` contains single-letter
        // note names. `candidates2` contains double-letter note names. We want to try all
        // single-letter note names first and favor them even over double-letter names that might be
        // closer. For example, if the scale has something a tiny bit sharper than `C`, we'd rather
        // just see `C` than something like `Cy`, even if `Cy` is a little bit closer. On the other
        // hand, if `Cy` is closest to some step but `Cx` is even closer to the same step, we'd
        // rather use `Cx`. Initialize `candidates1` with the candidate representing `A`, which will
        // always be a perfect match for the root pitch.
        let mut candidates1: Vec<Candidate> = vec![Candidate {
            closest_step: 0,
            delta: 0.0,
            note_path: NotePath {
                octaves: 0,
                step1: 1,
                step2: 0,
            },
        }];
        let mut candidates2: Vec<Candidate> = Default::default();
        let divided_interval_f64 = self.divided_interval.to_f64().unwrap();
        let max_octaves = divided_interval_f64.log2().ceil() as i32;
        // Consider larger intervals (lower letters) first...
        for step1 in 2..=25 {
            // ...and consider the "up" direction before the "down" direction.
            for neg1 in [false, true] {
                // Never consider `b` (which drops a whole octave). Also ignore `d` and `c` since
                // `Bd` = `C` and `Bc` = `D`.
                if neg1 && step1 < 5 {
                    continue;
                }
                // The loops so far give us `B`, `C`, `D`, `E`, `e`, `F`, `f`, `G`, `g`, etc.
                // Consider as many leading `B` notes as necessary to get us within the interval
                // range. This allows us to find things like `Bi` as a single-letter note for 8/9
                // and also allows us to find names for notes that are farther than an octave away
                // from the root on a scale that divides a larger interval. For example, `BE` would
                // be step 16 in 19-ED3, which is very close to just extending 12-EDO to an octave
                // and a fifth.
                for octaves in 0..=max_octaves {
                    if let Some(candidate) = NotePath::candidate(
                        octaves,
                        if neg1 { -step1 } else { step1 },
                        0,
                        divided_interval_f64,
                        divisions,
                    ) {
                        candidates1.push(candidate);
                    };
                    // Consider all the second step refinements of the single-letter notes.
                    if step1 == 2 {
                        // We don't need to consider two-latter cases starting with `B` -- those are
                        // automatically handled as single-letter cases since prepending `B` is free.
                        continue;
                    }
                    // Allow step 2 to be the same size or smaller than step 1. Never consider a
                    // step 2 of `b` (meaningless -- will always drop below the interval) or `c` or
                    // `d` since we'd prefer to go up by `C` than down by `d` or up by `D` than down
                    // by `c`. Notes like `II` (81/64) are valid, but any note where the second step
                    // is larger than the first would be picked up in the other direction. In other
                    // words, we don't need to look at `eC` -- we will find `Ce` instead.
                    for step2 in std::cmp::max(step1, 5)..=25 {
                        for neg2 in [false, true] {
                            if let Some(candidate) = NotePath::candidate(
                                octaves,
                                if neg1 { -step1 } else { step1 },
                                if neg2 { -step2 } else { step2 },
                                divided_interval_f64,
                                divisions,
                            ) {
                                candidates2.push(candidate);
                            };
                        }
                    }
                }
            }
        }
        // After we've identified all the candidates, iterate through them, tracking the best pitch
        // we've seen so far. See docs on `Candidate`.
        let mut winners: HashMap<i32, Candidate> = Default::default();
        for candidate in candidates1.into_iter().chain(candidates2) {
            match winners.entry(candidate.closest_step) {
                Entry::Occupied(mut e) => {
                    let old = e.get();
                    // If a single-letter note's closest step is this step, don't override that
                    // choice with a double-letter note. For example, if `C`'s closest step and
                    // `Cy`'s closest step are the same, we'd rather see `C` than `Cy`. Otherwise,
                    // replace notes with better candidates.
                    if (candidate.note_path.step2 == 0 || old.note_path.step2 != 0)
                        && candidate.closer_than(old)
                    {
                        e.insert(candidate);
                    }
                }
                Entry::Vacant(e) => {
                    e.insert(candidate);
                }
            }
        }

        // Generate notes and primary names based on the winning candidates.
        let mut primary_names = HashMap::new();
        let mut notes = HashMap::new();
        let num = *self.divided_interval.numer();
        let den = *self.divided_interval.denom();
        for step in 0..divisions {
            let mut name: Cow<str> = Cow::Owned(format!("A{step}"));
            let pitch = Pitch::new(vec![Factor::new(num, den, step, divisions).unwrap()]);
            notes.insert(name.clone(), pitch.clone());
            if let Some(candidate) = winners.get(&step) {
                name = candidate.name();
                notes.insert(name.clone(), pitch.clone());
            }
            primary_names.insert(pitch, name);
        }
        Assignments {
            notes,
            primary_names,
        }
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

    #[test]
    fn test_assign_notes() {
        // This is mostly tested through test17-generated.stq, which make a bunch of different
        // generated scales. This test can be used for manual debugging.
        let g = NoteGenerator {
            divisions: Some(12),
            divided_interval: Ratio::from_integer(2),
            tolerance: Default::default(),
        };
        let a = g.assign_generated_notes();
        println!("{a:?}");
        let exp: HashMap<Pitch, Cow<str>> = [
            ("1", "A"),
            ("^1|12", "R"),
            ("^2|12", "I"),
            ("^3|12", "F"),
            ("^4|12", "E"),
            ("^5|12", "D"),
            ("^6|12", "Cq"),
            ("^7|12", "C"),
            ("^8|12", "Be"),
            ("^9|12", "Bf"),
            ("^10|12", "Bi"),
            ("^11|12", "Br"),
        ]
        .into_iter()
        .map(|(pitch, name)| (Pitch::must_parse(pitch), Cow::Owned(name.to_string())))
        .collect();

        assert_eq!(a.primary_names, exp);
    }
}
