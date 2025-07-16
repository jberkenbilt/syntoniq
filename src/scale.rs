use crate::pitch::Pitch;
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Scale {
    pub name: String,
    pub tonic: Pitch,
    pub octave_steps: u8,
    pub step_factor: Pitch,
    pub note_names: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct Note {
    pub name: String,
    pub freq: f32,
    /// The midi number in adjusted_midi not based on pitch but rather based on scale degrees away
    /// from the tonic, which is always note 60. This allows us to send MIDI not numbers to a system
    /// like Surge-XT
    pub adjusted_midi: u8,
    /// This is the closest 12-TET midi number to the pitch and a pitch bend assuming ±2 cents.
    pub nearest_pitch_midi: (u8, u16),
    /// TODO: indicate whether close to one of the special just intonation intervals
    pub _closest_just_interval: Option<()>,
}

impl Scale {
    /// Return the frequency of the scale tone `octave` octaves and `step` steps above the tonic.
    /// Both values can be negative. This will panic or overflow if values are out of range.
    pub fn note(&self, octave: i8, step: i8) -> Note {
        let mut freq = self.tonic.as_float();
        freq *= 2.0f32.powf(octave as f32);
        freq *= self.step_factor.as_float().powf(step as f32);
        let pitch_midi = Self::freq_midi(freq);
        let adjusted_midi = (60 + self.octave_steps as i8 * octave + step) as u8;
        let note_idx = (step % self.octave_steps as i8) as usize;
        let name = if note_idx > self.note_names.len() {
            format!("{octave}.{step}")
        } else {
            self.note_names[note_idx].to_string()
        };
        Note {
            name,
            freq,
            adjusted_midi,
            nearest_pitch_midi: pitch_midi,
            _closest_just_interval: None,
        }
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
    use crate::pitch::{EqualDivision, Multiplier, Ratio};

    #[test]
    pub fn test_notes() -> anyhow::Result<()> {
        let edo12 = Scale {
            name: "edo-12".to_string(),
            tonic: Pitch {
                base: 261.626,
                multipliers: vec![],
            },
            octave_steps: 12,
            step_factor: Pitch {
                base: 1.0,
                multipliers: vec![Multiplier::EqualDivision(EqualDivision {
                    exp_numerator: 1,
                    exp_denominator: 12,
                    base_numerator: 2,
                    base_denominator: 1,
                })],
            },
            note_names: vec![],
        };
        let note = edo12.note(0, 9);
        dbg!(&note);
        assert_eq!(note.freq.round(), 440.0);
        assert_eq!(note.adjusted_midi, 69);
        assert_eq!(note.nearest_pitch_midi, (69, 8192));
        assert_eq!(note.name, "0.9");

        let edo6 = Scale {
            name: "edo-6".to_string(),
            tonic: Pitch {
                base: 440.0,
                multipliers: vec![Multiplier::Ratio(Ratio {
                    numerator: 3,
                    denominator: 5,
                })],
            },
            octave_steps: 6,
            step_factor: Pitch {
                base: 1.0,
                multipliers: vec![Multiplier::EqualDivision(EqualDivision {
                    exp_numerator: 1,
                    exp_denominator: 6,
                    base_numerator: 2,
                    base_denominator: 1,
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
}
