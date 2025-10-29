use crate::events::{Color, Event, LightEvent, LightMode};
use crate::layout;
use anyhow::{anyhow, bail};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use syntoniq_common::pitch::{Factor, Pitch};

#[derive(Clone, Debug, PartialEq)]
pub struct Scale {
    pub name: String,
    pub scale_type: ScaleType,
    pub base_pitch: Pitch,
    pub orig_base_pitch: Pitch,
    pub note_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScaleDescription {
    // Keep fields in order for sorting
    pub name: String,
    pub orig_base_pitch: Pitch,
    pub base_pitch: Pitch,
}
impl Display for ScaleDescription {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = &self.name;
        let base_pitch = &self.base_pitch;
        let orig_base_pitch = &self.orig_base_pitch;
        write!(f, "{name}, base={base_pitch}")?;
        if base_pitch != orig_base_pitch {
            let factor = base_pitch / orig_base_pitch;
            write!(f, " (transposition: {orig_base_pitch} × {factor})")?;
        }
        Ok(())
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum ScaleType {
    EqualDivision(EqualDivision),
    Generic(GenericScale),
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct EqualDivision {
    /// divisions, interval numerator, interval denominator, e.g. (31, 2, 1) for 31-EDO
    pub divisions: (u32, u32, u32),
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct GenericScale {
    pub pitches: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct Note {
    pub name: String,
    pub description: String,
    pub pitch: Pitch,
    pub scale_description: ScaleDescription,
    /// Factor to multiply by base, useful for transcription
    pub base_factor: Pitch,
    pub colors: (Color, Color), // note off, note on
}
impl Note {
    pub fn light_event(&self, position: u8, velocity: u8) -> Event {
        let color = if velocity == 0 {
            self.colors.0
        } else {
            self.colors.1
        };
        Event::Light(LightEvent {
            mode: LightMode::On,
            position,
            color,
            label1: self.name.clone(),
            label2: self.description.clone(),
        })
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Note {
            name,
            description,
            pitch: _,
            scale_description,
            base_factor,
            colors: _,
        } = self;
        write!(
            f,
            "Note: {name} ({description}), pitch=base*{base_factor}, scale={scale_description}"
        )
    }
}

impl Scale {
    pub fn validate(&self) -> anyhow::Result<()> {
        match &self.scale_type {
            ScaleType::EqualDivision(ed) => {
                let (steps, num, den) = ed.divisions;
                if den == 0 || num == den || steps < 2 {
                    bail!(
                        "scale divisions for {}: {steps},{num},{den} can't generate a scale",
                        self.name,
                    );
                }
            }
            ScaleType::Generic(g) => {
                if g.pitches.len() != 64 {
                    bail!("exactly 64 pitches must be given for generic scales");
                }
                if self.note_names.len() != 64 && !self.note_names.is_empty() {
                    bail!("note names must be empty or contain 64 values");
                }
            }
        }
        Ok(())
    }

    pub fn description(&self) -> ScaleDescription {
        ScaleDescription {
            name: self.name.clone(),
            base_pitch: self.base_pitch.clone(),
            orig_base_pitch: self.orig_base_pitch.clone(),
        }
    }

    /// Return the frequency of the scale tone `cycle` cycles and `step` steps above the base pitch.
    /// Both values can be negative. This will panic or overflow if values are out of range. For
    /// divisions of the octave, `cycle` is an octave. Some scale types may not have cycles.
    pub fn edo_note(&self, cycle: i8, step: i8) -> Note {
        let ScaleType::EqualDivision(data) = &self.scale_type else {
            panic!("Scale::edo_note called for non-EDO scale");
        };
        self.note_equal_division(data, cycle, step)
    }

    pub fn generic_note(
        &self,
        cache: &mut HashMap<i8, Option<Arc<Note>>>,
        g: &GenericScale,
        row: i8,
        col: i8,
    ) -> anyhow::Result<Option<Arc<Note>>> {
        // The note names and pitches array are 81..88, 71..78, ...
        let idx = (layout::NOTE_ROWS - row) * layout::NOTE_COLS + col - 1;
        // Size of pitches was checked in config
        let pitch_str = g
            .pitches
            .get(idx as usize)
            // not possible -- validated in config
            .ok_or(anyhow!(
                "{}: pitches does not have enough elements",
                self.name
            ))?;
        if pitch_str.is_empty() {
            return Ok(None);
        }
        let position = 10 * row + col;
        if let Some(entry) = cache.get(&position).cloned() {
            // Previously computed value
            let value = entry.ok_or(anyhow!(
                "{}: loop detected at {position} while computing pitches",
                self.name
            ))?;
            return Ok(Some(value));
        }
        // Insert None for loop detection
        cache.insert(position, None);
        let error_prefix = format!(
            "{}: invalid syntax at row {row} ({} is top), col {col}",
            self.name,
            layout::NOTE_ROWS,
        );
        let (base_pitch, factor) = if pitch_str.starts_with("[") {
            // This is relative to another cell
            let fields: Vec<&str> = pitch_str.splitn(2, "]").collect();
            let factor = fields.get(1).ok_or(anyhow!("{error_prefix}"))?.to_string();
            let other = &fields[0][1..];
            let other_pos: i8 = other
                .parse()
                .map_err(|e| anyhow!("{error_prefix}: invalid other cell {other}: {e}"))?;
            let other_row = other_pos / 10;
            let other_col = other_pos % 10;
            let Some(other_note) = self.generic_note(cache, g, other_row, other_col)? else {
                bail!("{error_prefix}: referenced position {other_pos} is empty");
            };
            (other_note.pitch.clone(), factor)
        } else {
            (self.base_pitch.clone(), pitch_str.clone())
        };
        let factor_pitch = Pitch::parse(&factor)?;
        let pitch = &base_pitch * &factor_pitch;
        let name = self
            .note_names
            .get(idx as usize)
            .unwrap_or(&factor)
            .to_string();
        let colors = Self::interval_color(pitch.as_float() / self.base_pitch.as_float());
        let description = pitch_str.to_string();
        let note = Arc::new(Note {
            name,
            description,
            base_factor: &pitch / &self.base_pitch,
            pitch,
            scale_description: self.description(),
            colors,
        });
        cache.insert(position, Some(note.clone()));
        Ok(Some(note))
    }

    pub fn note_equal_division(&self, data: &EqualDivision, mut cycle: i8, mut step: i8) -> Note {
        let (divisions, num, den) = data.divisions;
        while step < 0 {
            step += divisions as i8;
            cycle -= 1
        }
        let steps = divisions as i32 * cycle as i32 + step as i32;
        let pitch = self.base_pitch.concat(&Pitch::new(vec![
            Factor::new(num, den, steps, divisions as i32).unwrap(),
        ]));
        let freq = pitch.as_float();
        let normalized_step = step % divisions as i8;
        let note_idx = normalized_step as usize;
        let name = self.note_names.get(note_idx).cloned().unwrap_or_default();
        let colors = if normalized_step == 1 {
            // Special case: use a slightly different color for idx 1 so we can see clearly
            // where the single step is.
            (Color::SingleStepOff, Color::SingleStepOn)
        } else {
            Self::interval_color(freq / self.base_pitch.as_float())
        };
        let description = format!("{cycle}.{step}");
        Note {
            name,
            description,
            base_factor: &pitch / &self.base_pitch,
            pitch,
            scale_description: self.description(),
            colors,
        }
    }

    fn interval_color(mut interval: f32) -> (Color, Color) {
        while interval <= 1.0 {
            interval *= 2.0;
        }
        while interval > 2.0 {
            interval /= 2.0;
        }
        // If the color is very close to of the 5-limit Just Intonation ratios below or their
        // reciprocals, assign a color. Otherwise, assign a default.
        // Note: 12-EDO minor third is by 15.64 cents.
        let tolerance_cents = 2.0f32.powf(16.0 / 1200.0);
        for (ratio, colors) in [
            (1.0, (Color::TonicOff, Color::TonicOn)),
            (3.0 / 2.0, (Color::FifthOff, Color::FifthOn)),
            (5.0 / 4.0, (Color::MajorThirdOff, Color::MajorThirdOn)),
            (6.0 / 5.0, (Color::MinorThirdOff, Color::MinorThirdOn)),
        ] {
            // Interval will never be zero unless someone put zeros in their scale files, and we
            // check against that when validating the config file.
            for target in [ratio, 2.0 / ratio] {
                let difference = if interval > target {
                    interval / target
                } else {
                    target / interval
                };
                if difference < tolerance_cents {
                    return colors;
                }
            }
        }
        (Color::OtherOff, Color::OtherOn)
    }

    pub fn transpose(&mut self, amount: &Pitch) {
        self.base_pitch *= amount;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syntoniq_common::pitch::Factor;

    #[test]
    pub fn test_notes() -> anyhow::Result<()> {
        let base_pitch = Pitch::new(vec![Factor::new(261626, 1000, 1, 1)?]);
        let edo12 = Scale {
            name: "edo-12".to_string(),
            scale_type: ScaleType::EqualDivision(EqualDivision {
                divisions: (12, 2, 1),
            }),
            orig_base_pitch: base_pitch.clone(),
            base_pitch,
            note_names: vec![],
        };
        let note = edo12.edo_note(0, 9);
        assert_eq!(note.pitch.as_float().round(), 440.0);
        assert_eq!(note.pitch.midi().unwrap(), (69, 8192));
        assert_eq!(note.name, "");
        assert_eq!(note.description, "0.9");
        assert_eq!(note.colors, (Color::MinorThirdOff, Color::MinorThirdOn));

        let base_pitch = Pitch::new(vec![Factor::new(440, 1, 1, 1)?, Factor::new(3, 5, 1, 1)?]);
        let edo6 = Scale {
            name: "edo-6".to_string(),
            scale_type: ScaleType::EqualDivision(EqualDivision {
                divisions: (6, 2, 1),
            }),
            orig_base_pitch: base_pitch.clone(),
            base_pitch,
            note_names: ["C", "D", "E", "F#", "G#", "A#"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        };
        let note = edo6.edo_note(0, 3);
        assert_eq!((100.0 * note.pitch.as_float()).round(), 37335.0);
        assert_eq!(note.pitch.midi().unwrap(), (66, 8219));
        assert_eq!(note.name, "F#");

        Ok(())
    }

    #[test]
    fn test_interval_colors() {
        fn get_color(pitch: &str) -> Color {
            let (c, _) = Scale::interval_color(Pitch::must_parse(pitch).as_float());
            c
        }
        assert_eq!(get_color("1*3/2"), Color::FifthOff); // JI 5th
        assert_eq!(get_color("1*^9|12"), Color::MinorThirdOff); // 12-EDO major sixth
        assert_eq!(get_color("1*^10|31"), Color::MajorThirdOff); // 31-EDO major third
        assert_eq!(get_color("1*^7|17"), Color::FifthOff); // 17-EDO fourth
        assert_eq!(get_color("1*^5|17"), Color::OtherOff); // nope
    }
}
