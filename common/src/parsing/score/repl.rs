use crate::parsing::diagnostics::{Diagnostics, code};
use crate::parsing::model::{Span, Spanned, Token};
use crate::parsing::pass1::Parser1Intermediate;
use crate::parsing::score::generator::{Divisions, NoteGenerator};
use crate::parsing::score::{Generator, generator};
use crate::parsing::{pass1, score_helpers};
use crate::pitch::Pitch;
use num_rational::Ratio;
use serde::Serialize;
use winnow::combinator::{alt, delimited, fail, opt, preceded};
use winnow::stream::AsChar;
use winnow::token::take_while;
use winnow::{LocatingSlice, Parser};

// Leverage of the pass1 parsing machinery to parse commands from the REPL used by
// syntoniq-kbd prompt.

#[derive(Serialize, Debug, Clone, Default, PartialEq)]
pub struct ReplNote {
    name: String,
    pitch: Pitch,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct DivisionsAndCycle {
    divisions: Divisions,
    cycle: Ratio<u32>,
}
impl Default for DivisionsAndCycle {
    fn default() -> Self {
        Self {
            divisions: Default::default(),
            cycle: Ratio::from_integer(2),
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum PromptCommand {
    Reset,
    Clear,
    SetDivisions {
        divisions: Divisions,
    },
    SetCycleRatio {
        cycle: Ratio<u32>,
    },
    SetBaseAbsolute {
        pitch: Pitch,
    },
    SetBaseRelative {
        pitch: Pitch,
    },
    Transpose {
        pitch_from: ReplNote,
        written: ReplNote,
    },
    Play {
        n: Option<u8>,
        note: Option<ReplNote>,
    },
}
type ReplToken<'s> = Spanned<Token<'s, PromptCommand>>;
trait ReplParser<'s>: Parser1Intermediate<'s, ReplToken<'s>> {}
impl<'s, P: Parser1Intermediate<'s, ReplToken<'s>>> ReplParser<'s> for P {}

fn parse_repl<'s, P, F, T>(p: P, f: F) -> impl ReplParser<'s>
where
    P: Parser1Intermediate<'s, T>,
    F: Fn(&'s str, Span, T) -> PromptCommand,
{
    pass1::parse1_intermediate(p, move |raw, span, out| {
        Token::new_spanned(raw, span, f(raw, span, out))
    })
}

fn octave<'s>(diags: &Diagnostics) -> impl Parser1Intermediate<'s, Spanned<i8>> {
    pass1::parse1_intermediate(
        (
            alt((',', '\'', fail))
                .with_span()
                .map(|(ch, r)| Spanned::<char>::new(r, ch)),
            opt(pass1::number_intermediate(diags)),
        ),
        |_raw, _span, (sym, num)| score_helpers::check_note_octave(diags, sym, num),
    )
}

fn note<'s>(diags: &Diagnostics, dc: &DivisionsAndCycle) -> impl Parser1Intermediate<'s, ReplNote> {
    pass1::parse1_intermediate(
        (
            (
                take_while(1, |c: char| AsChar::is_alpha(c)),
                take_while(0.., |x: char| {
                    AsChar::is_alphanum(x) || "#%+-!/".contains(x)
                }),
            )
                .with_span()
                .map(|((first, rest), r)| Spanned::<String>::new(r, format!("{first}{rest}"))),
            opt(octave(diags)),
        ),
        |raw, _span, (name, octave)| to_repl_note(diags, dc, name, octave, raw),
    )
}

fn to_repl_note(
    diags: &Diagnostics,
    dc: &DivisionsAndCycle,
    name: Spanned<String>,
    octave: Option<Spanned<i8>>,
    raw: &str,
) -> ReplNote {
    let g = NoteGenerator {
        divisions: dc.divisions.divisions,
        divided_interval: dc.divisions.interval,
        tolerance: Default::default(),
    };
    match g.get_note(diags, &name.as_ref()) {
        None => {
            diags.err(code::SYNTAX, name.span, "invalid generated note");
            Default::default()
        }
        Some(p) => {
            // The logic of multiplying by the cycle offset is duplicated in various places.
            let pitch = match octave {
                None => p,
                Some(count) => &p * &Pitch::from(dc.cycle.pow(count.value as i32)),
            };
            ReplNote {
                name: raw.to_string(),
                pitch,
            }
        }
    }
}

fn reset<'s>() -> impl ReplParser<'s> {
    parse_repl("!!!", |_raw, _span, _out| PromptCommand::Reset)
}

fn clear<'s>() -> impl ReplParser<'s> {
    parse_repl("!!", |_raw, _span, _out| PromptCommand::Clear)
}

fn set_divisions<'s>(diags: &Diagnostics) -> impl ReplParser<'s> {
    parse_repl(generator::step(diags), |_raw, _span, step| {
        PromptCommand::SetDivisions {
            divisions: step.to_divisions(),
        }
    })
}

fn set_cycle_ratio<'s>(diags: &Diagnostics) -> impl ReplParser<'s> {
    parse_repl(
        preceded(
            ('%', opt(pass1::space())),
            (
                pass1::number_intermediate(diags),
                opt(preceded('/', pass1::number_intermediate(diags))),
            ),
        ),
        |_raw, _span, (a, maybe_b)| {
            let ratio = Ratio::new(a.value, maybe_b.map(Spanned::value).unwrap_or(1));
            PromptCommand::SetCycleRatio { cycle: ratio }
        },
    )
}

fn set_base<'s>(diags: &Diagnostics) -> impl ReplParser<'s> {
    parse_repl(
        (
            alt(('=', '*', fail)),
            opt(pass1::space()),
            take_while(1.., |c| !AsChar::is_space(c)).with_span(),
        ),
        |_raw, _span, (ch, _, (rest, rest_span))| {
            let rest_span = Span::from(rest_span);
            let pitch = match Pitch::parse(rest) {
                Ok(p) => p,
                Err(e) => {
                    diags.err(code::SYNTAX, rest_span, e.to_string());
                    Pitch::default()
                }
            };
            if ch == '=' {
                PromptCommand::SetBaseAbsolute { pitch }
            } else {
                debug_assert_eq!(ch, '*');
                PromptCommand::SetBaseRelative { pitch }
            }
        },
    )
}

fn play_n<'s>(diags: &Diagnostics, dc: &DivisionsAndCycle) -> impl ReplParser<'s> {
    parse_repl(
        (
            pass1::number_intermediate(diags),
            opt(pass1::space()),
            '<',
            opt(pass1::space()),
            opt(note(diags, dc)),
        ),
        |_raw, _span, (num, _, _, _, note)| {
            let n = match u8::try_from(num.value) {
                Ok(n) => n,
                Err(_) => {
                    diags.err(code::SYNTAX, num.span, "note number is out of range");
                    0
                }
            };
            PromptCommand::Play { n: Some(n), note }
        },
    )
}

fn play_bare<'s>(diags: &Diagnostics, dc: &DivisionsAndCycle) -> impl ReplParser<'s> {
    parse_repl(note(diags, dc), |_raw, _span, note| PromptCommand::Play {
        n: None,
        note: Some(note),
    })
}

fn transpose<'s>(diags: &Diagnostics, dc: &DivisionsAndCycle) -> impl ReplParser<'s> {
    parse_repl(
        (
            note(diags, dc),
            opt(pass1::space()),
            '>',
            opt(pass1::space()),
            note(diags, dc),
        ),
        |_raw, _span, (pitch_from, _, _, _, written)| PromptCommand::Transpose {
            pitch_from,
            written,
        },
    )
}

fn command<'s>(diags: &Diagnostics, dc: &DivisionsAndCycle) -> impl ReplParser<'s> {
    delimited(
        opt(pass1::space()),
        alt((
            reset(),
            clear(),
            set_divisions(diags),
            set_cycle_ratio(diags),
            transpose(diags, dc),
            set_base(diags),
            play_n(diags, dc),
            play_bare(diags, dc),
        )),
        opt(pass1::space()),
    )
}

pub fn parse_repl_line(line: &str, dc: &DivisionsAndCycle) -> Result<PromptCommand, Diagnostics> {
    let input = LocatingSlice::new(line);
    let diags = Diagnostics::new();
    let r = command(&diags, dc).parse(input);
    match r {
        Ok(t) => {
            if diags.has_errors() {
                Err(diags)
            } else {
                Ok(t.value.t)
            }
        }
        Err(e) => {
            diags.err(
                code::SYNTAX,
                0..line.len(),
                format!("error parsing command: {e}"),
            );
            Err(diags)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::score;

    #[test]
    fn test_note() {
        fn parse_note(
            divisions: u32,
            divided_interval: Ratio<u32>,
            cycle: Ratio<u32>,
            s: &str,
        ) -> Pitch {
            let dc = DivisionsAndCycle {
                divisions: Divisions {
                    interval: divided_interval,
                    divisions: if divisions == 0 {
                        None
                    } else {
                        Some(divisions)
                    },
                },
                cycle,
            };
            let diags = Diagnostics::new();
            let input = LocatingSlice::new(s);
            note(&diags, &dc).parse(input).unwrap().pitch
        }
        assert_eq!(
            parse_note(0, Ratio::from_integer(2), Ratio::from_integer(2), "A"),
            Default::default(),
        );
        assert_eq!(
            parse_note(0, Ratio::from_integer(2), Ratio::from_integer(2), "C"),
            Pitch::must_parse("3/2"),
        );
        assert_eq!(
            parse_note(12, Ratio::from_integer(2), Ratio::from_integer(2), "C"),
            Pitch::must_parse("^7|12"),
        );
        assert_eq!(
            parse_note(17, Ratio::from_integer(3), Ratio::from_integer(5), "C!"),
            Pitch::must_parse("3/2"),
        );
        assert_eq!(
            parse_note(0, Ratio::from_integer(3), Ratio::from_integer(2), "C'"),
            Pitch::must_parse("3"),
        );
        assert_eq!(
            parse_note(0, Ratio::from_integer(3), Ratio::from_integer(2), "E,2"),
            Pitch::must_parse("5/16"),
        );
    }

    #[test]
    fn test_commands() {
        let dc = DivisionsAndCycle::default();
        assert_eq!(parse_repl_line(" !!! ", &dc).unwrap(), PromptCommand::Reset);
        assert_eq!(parse_repl_line(" !! ", &dc).unwrap(), PromptCommand::Clear);
        assert_eq!(
            parse_repl_line(" ! ", &dc).unwrap(),
            PromptCommand::SetDivisions {
                divisions: Divisions::default()
            }
        );
        assert_eq!(
            parse_repl_line("!3", &dc).unwrap(),
            PromptCommand::SetDivisions {
                divisions: Divisions {
                    interval: Ratio::from_integer(2),
                    divisions: Some(3),
                }
            }
        );
        assert_eq!(
            parse_repl_line("!3/4", &dc).unwrap(),
            PromptCommand::SetDivisions {
                divisions: Divisions {
                    interval: Ratio::from_integer(3),
                    divisions: Some(4),
                }
            }
        );
        assert_eq!(
            parse_repl_line("!3/0", &dc).unwrap(),
            PromptCommand::SetDivisions {
                divisions: Divisions {
                    interval: Ratio::from_integer(3),
                    divisions: None,
                }
            }
        );
        assert_eq!(
            parse_repl_line("!3/1", &dc).unwrap(),
            PromptCommand::SetDivisions {
                divisions: Divisions {
                    interval: Ratio::from_integer(3),
                    divisions: None,
                }
            }
        );
        assert_eq!(
            parse_repl_line("!3/2/0", &dc).unwrap(),
            PromptCommand::SetDivisions {
                divisions: Divisions {
                    interval: Ratio::new(3, 2),
                    divisions: None,
                }
            }
        );
        assert_eq!(
            parse_repl_line("!3/4/5", &dc).unwrap(),
            PromptCommand::SetDivisions {
                divisions: Divisions {
                    interval: Ratio::new(3, 4),
                    divisions: Some(5),
                }
            }
        );
        assert_eq!(
            parse_repl_line("% 2", &dc).unwrap(),
            PromptCommand::SetCycleRatio {
                cycle: Ratio::from_integer(2)
            }
        );
        assert_eq!(
            parse_repl_line("% 5/4", &dc).unwrap(),
            PromptCommand::SetCycleRatio {
                cycle: Ratio::new(5, 4)
            }
        );
        assert_eq!(
            parse_repl_line(" a > b ", &dc).unwrap(),
            PromptCommand::Transpose {
                pitch_from: ReplNote {
                    name: "a".to_string(),
                    pitch: Default::default(),
                },

                written: ReplNote {
                    name: "b".to_string(),
                    pitch: Pitch::must_parse("1/2"),
                },
            }
        );
        assert_eq!(
            parse_repl_line(" = 220*^3|4 ", &dc).unwrap(),
            PromptCommand::SetBaseAbsolute {
                pitch: Pitch::must_parse("220*^3|4")
            }
        );
        assert_eq!(
            parse_repl_line("*264*5/3 ", &dc).unwrap(),
            PromptCommand::SetBaseRelative {
                pitch: Pitch::must_parse("440")
            }
        );
        assert_eq!(
            parse_repl_line(" JI ", &dc).unwrap(),
            PromptCommand::Play {
                n: None,
                note: Some(ReplNote {
                    name: "JI".to_string(),
                    pitch: Pitch::must_parse("5/4"),
                })
            }
        );
        let div_12edo = DivisionsAndCycle {
            divisions: Divisions {
                interval: Ratio::from_integer(2),
                divisions: Some(12),
            },
            cycle: Ratio::from_integer(2),
        };
        assert_eq!(
            parse_repl_line(" C ", &div_12edo).unwrap(),
            PromptCommand::Play {
                n: None,
                note: Some(ReplNote {
                    name: "C".to_string(),
                    pitch: Pitch::must_parse("^7|12"),
                })
            }
        );
        let div_27ed3 = DivisionsAndCycle {
            divisions: Divisions {
                interval: Ratio::from_integer(3),
                divisions: Some(27),
            },
            cycle: Ratio::from_integer(5),
        };
        assert_eq!(
            parse_repl_line(" 4<JK ", &div_27ed3).unwrap(),
            PromptCommand::Play {
                n: Some(4),
                note: Some(ReplNote {
                    name: "JK".to_string(),
                    pitch: Pitch::must_parse("3^5|27"),
                }),
            }
        );
        assert_eq!(
            parse_repl_line(" 4<JK' ", &div_27ed3).unwrap(),
            PromptCommand::Play {
                n: Some(4),
                note: Some(ReplNote {
                    name: "JK'".to_string(),
                    pitch: Pitch::must_parse("5*3^5|27"),
                }),
            }
        );
        assert_eq!(
            parse_repl_line(" 4<JK! ", &div_27ed3).unwrap(),
            PromptCommand::Play {
                n: Some(4),
                note: Some(ReplNote {
                    name: "JK!".to_string(),
                    pitch: Pitch::must_parse("11/9"),
                }),
            }
        );
        assert_eq!(
            parse_repl_line(" 4<JK!,2 ", &div_27ed3).unwrap(),
            PromptCommand::Play {
                n: Some(4),
                note: Some(ReplNote {
                    name: "JK!,2".to_string(),
                    pitch: Pitch::must_parse("11/9*1/25"),
                }),
            }
        );
        assert_eq!(
            parse_repl_line(" 4<JK!31 ", &div_27ed3).unwrap(),
            PromptCommand::Play {
                n: Some(4),
                note: Some(ReplNote {
                    name: "JK!31".to_string(),
                    pitch: Pitch::must_parse("^9|31"),
                }),
            }
        );
        assert_eq!(
            parse_repl_line("5<", &div_27ed3).unwrap(),
            PromptCommand::Play {
                n: Some(5),
                note: None,
            }
        );
    }

    #[test]
    fn test_errors() {
        let dc = DivisionsAndCycle::default();
        assert!(
            parse_repl_line(" !!!! ", &dc)
                .unwrap_err()
                .to_string()
                .contains("error parsing command")
        );
        assert!(
            parse_repl_line("E?", &dc)
                .unwrap_err()
                .to_string()
                .contains("error parsing command")
        );
        assert!(
            parse_repl_line("% 2/3/4", &dc)
                .unwrap_err()
                .to_string()
                .contains("error parsing command")
        );
        assert!(
            parse_repl_line("500 < E", &dc)
                .unwrap_err()
                .to_string()
                .contains("note number is out of range")
        );
        assert!(
            parse_repl_line("5 < A4", &dc)
                .unwrap_err()
                .to_string()
                .contains("invalid generated note")
        );
        assert!(
            parse_repl_line("= a!b!c", &dc)
                .unwrap_err()
                .to_string()
                .contains("unable to parse as pitch")
        );
    }

    #[test]
    fn test_through_score() {
        let dc = DivisionsAndCycle::default();
        assert!(score::parse_prompt_line("A17", &dc).is_none());
        assert!(score::parse_prompt_line("E", &dc).is_some());
    }
}
