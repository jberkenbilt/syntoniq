use crate::events::Color;
use crate::pitch::Pitch;
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Scale {
    pub name: String,
    #[serde(flatten)]
    pub scale_type: ScaleType,
    pub base_pitch: Pitch,
    pub note_names: Vec<String>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum ScaleType {
    EqualDivision(EqualDivision),
    _KeepClippyQuiet, // TODO: remove when we add a second type
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct EqualDivision {
    /// divisions, interval numerator, interval denominator, e.g. (31, 2, 1) for EDO-31
    pub divisions: (i8, i8, i8),
}

#[derive(Debug, PartialEq)]
pub struct Note {
    pub name: String,
    pub scale_name: String,
    pub cycle: i8,
    pub step: i8,
    pub freq: f32,
    /// The midi number in adjusted_midi not based on pitch but rather based on scale degrees away
    /// from the tonic, which is always note 60. This allows us to send MIDI not numbers to a system
    /// like Surge-XT
    pub adjusted_midi: u8,
    /// This is the closest 12-TET midi number to the pitch and a pitch bend assuming ±2 cents.
    pub nearest_pitch_midi: (u8, u16),
    pub colors: (Color, Color), // note off, note on
}

impl Scale {
    /// Return the frequency of the scale tone `cycle` cycles and `step` steps above the base pitch.
    /// Both values can be negative. This will panic or overflow if values are out of range. For
    /// divisions of the octave, `cycle` is an octave. Some scale types may not have cycles.
    pub fn note(&self, cycle: i8, step: i8) -> Note {
        match &self.scale_type {
            ScaleType::EqualDivision(data) => self.note_equal_division(data, cycle, step),
            ScaleType::_KeepClippyQuiet => unreachable!(),
        }
    }

    pub fn note_equal_division(&self, data: &EqualDivision, cycle: i8, step: i8) -> Note {
        let mut freq = self.base_pitch.as_float();
        let base = freq;
        let (divisions, num, den) = data.divisions;
        let interval = num as f32 / den as f32;
        freq *= interval.powf(cycle as f32);
        let step_factor = interval.powf(1.0 / divisions as f32);
        freq *= step_factor.powf(step as f32);
        let pitch_midi = Self::freq_midi(freq);
        let adjusted_midi = (60 + divisions * cycle + step) as u8;
        let note_idx = (step % divisions) as usize;
        let name = self.note_names.get(note_idx).cloned().unwrap_or_default();
        let colors = Self::interval_color(freq / base);
        Note {
            name,
            scale_name: self.name.clone(),
            cycle,
            step,
            freq,
            adjusted_midi,
            nearest_pitch_midi: pitch_midi,
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
        // Note: EDO-12 minor third is by 15.64 cents.
        let tolerance_cents = 2.0f32.powf(16.0 / 1200.0);
        for (ratio, colors) in [
            (1.0, (Color::Cyan, Color::Yellow)),
            (3.0 / 2.0, (Color::Blue, Color::Green)),
            (5.0 / 4.0, (Color::Purple, Color::Pink)),
            (6.0 / 5.0, (Color::Red, Color::Orange)),
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
        (Color::Gray, Color::White)
    }

    /// Compute a frequency to a midi note number and a pitch bend value using ±2 semitones.
    /// Panics if the frequency is out of range.
    fn freq_midi(f: f32) -> (u8, u16) {
        let n1 = 69.0 + 12.0 * (f / 440.0).log2();
        let note = n1.round() as u8;
        let delta = n1 - note as f32;
        // - pitch bend is 8192 + 8192 * (semitones/bend range)
        // - bend range is typically 2 semitones
        // - 8192*delta/2 is 4096*delta
        // In other words, this the fraction numerator centered at 8192.
        let bend = 8192 + (4096.0 * delta).round() as u16;
        (note, bend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitch::{Multiplier, Ratio};

    #[test]
    pub fn test_notes() -> anyhow::Result<()> {
        let edo12 = Scale {
            name: "edo-12".to_string(),
            scale_type: ScaleType::EqualDivision(EqualDivision {
                divisions: (12, 2, 1),
            }),
            base_pitch: Pitch {
                base: 261.626,
                multipliers: vec![],
            },
            note_names: vec![],
        };
        let note = edo12.note(0, 9);
        dbg!(&note);
        assert_eq!(note.freq.round(), 440.0);
        assert_eq!(note.adjusted_midi, 69);
        assert_eq!(note.nearest_pitch_midi, (69, 8192));
        assert_eq!(note.name, "");
        assert_eq!(note.cycle, 0);
        assert_eq!(note.step, 9);
        assert_eq!(note.colors, (Color::Red, Color::Orange));

        let edo6 = Scale {
            name: "edo-6".to_string(),
            scale_type: ScaleType::EqualDivision(EqualDivision {
                divisions: (6, 2, 1),
            }),
            base_pitch: Pitch {
                base: 440.0,
                multipliers: vec![Multiplier::Ratio(Ratio {
                    numerator: 3,
                    denominator: 5,
                })],
            },
            note_names: ["C", "D", "E", "F#", "G#", "A#"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        };
        let note = edo6.note(0, 3);
        dbg!(&note);
        assert_eq!((100.0 * note.freq).round(), 37335.0);
        assert_eq!(note.adjusted_midi, 63);
        assert_eq!(note.nearest_pitch_midi, (66, 8833));
        assert_eq!(note.name, "F#");

        Ok(())
    }

    #[test]
    fn test_interval_colors() {
        fn get_color(pitch: &str) -> Color {
            let (c, _) = Scale::interval_color(Pitch::parse(pitch).unwrap().as_float());
            c
        }
        assert_eq!(get_color("1*3/2"), Color::Blue); // JI 5th
        assert_eq!(get_color("1*9\\12"), Color::Red); // EDO-12 minor sixth
        assert_eq!(get_color("1*10\\31"), Color::Purple); // EDO-31 major third
        assert_eq!(get_color("1*7\\17"), Color::Blue); // EDO-17 fourth
        assert_eq!(get_color("1*5\\17"), Color::Gray); // nope
    }
}
