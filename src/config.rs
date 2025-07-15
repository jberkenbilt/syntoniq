use crate::scale::Scale;
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
struct Config {
    scale: Vec<Scale>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitch::{EqualDivision, Multiplier, Pitch};

    #[test]
    fn test_toml() {
        const CONFIG: &str = r#"
[[scale]]
name = "EDO-12"
tonic = "220*3\\12"
octave_steps = 12
step_factor = "*1\\12"
note_names = ["C", "C♯", "D", "E♭", "E", "F", "F♯", "G", "A♭", "A", "B♭", "B"]
"#;
        let exp = Config {
            scale: vec![Scale {
                name: "EDO-12".to_string(),
                tonic: Pitch {
                    base: 220.0,
                    multipliers: vec![Multiplier::EqualDivision(EqualDivision {
                        exp_numerator: 3,
                        exp_denominator: 12,
                        base_numerator: 2,
                        base_denominator: 1,
                    })],
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
                note_names: [
                    "C", "C♯", "D", "E♭", "E", "F", "F♯", "G", "A♭", "A", "B♭", "B",
                ]
                .into_iter()
                .map(str::to_string)
                .collect(),
            }],
        };
        let c: Config = toml::from_str(CONFIG).unwrap();
        assert_eq!(c, exp);
    }
}
