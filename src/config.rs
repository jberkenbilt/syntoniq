use crate::layout::Layout;
use crate::scale::{Scale, ScaleType};
use anyhow::anyhow;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Deserialize, Debug, PartialEq)]
struct ConfigFile {
    scale: Vec<Scale>,
    layout: Vec<Layout>,
}

pub struct Config {
    pub layouts: Vec<Arc<Layout>>,
}

impl Config {
    pub fn load(file: PathBuf) -> anyhow::Result<Self> {
        let data = fs::read(&file)?;
        let c: ConfigFile = toml::from_slice(&data)?;
        let mut scales_by_name = HashMap::new();
        for scale in c.scale {
            let name = scale.name.clone();
            match &scale.scale_type {
                ScaleType::EqualDivision(ed) => {
                    let (steps, num, den) = ed.divisions;
                    if den == 0 || num == den || steps < 2 {
                        return Err(anyhow!(
                            "scale divisions for {name}: {steps},{num},{den} can't generate a scale"
                        ));
                    }
                }
            }
            if scales_by_name
                .insert(name.clone(), Arc::new(scale))
                .is_some()
            {
                return Err(anyhow!("duplicated scale {}", name));
            }
        }
        let mut layouts = Vec::new();
        for mut layout in c.layout.into_iter() {
            let Some(scale) = scales_by_name.get(&layout.scale_name) else {
                return Err(anyhow!(
                    "layout {}: no scale {}",
                    layout.name,
                    layout.scale_name
                ));
            };
            layout.scale = Some(scale.clone());
            layouts.push(Arc::new(layout));
        }
        Ok(Config { layouts })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitch::{Exponent, Multiplier, Pitch};
    use crate::scale::{EqualDivision, ScaleType};

    #[test]
    fn test_toml() {
        const CONFIG: &str = r#"
[[scale]]
name = "EDO-12"
type = "EqualDivision"
divisions = [12, 2, 1]
base_pitch = "220*3\\12" # middle C for A-440 12-TET scale
note_names = ["C", "C♯", "D", "E♭", "E", "F", "F♯", "G", "A♭", "A", "B♭", "B"]
[[layout]]
name = "5x3"
bbox = [1, 1, 8, 8]
base = [2, 2]
scale_name = "EDO-12"
steps = [2, 1]
"#;
        let exp = ConfigFile {
            scale: vec![Scale {
                name: "EDO-12".to_string(),
                base_pitch: Pitch {
                    base: 220.0,
                    multipliers: vec![Multiplier::Exponent(Exponent {
                        exp_numerator: 3,
                        exp_denominator: 12,
                        base_numerator: 2,
                        base_denominator: 1,
                    })],
                },
                scale_type: ScaleType::EqualDivision(EqualDivision {
                    divisions: (12, 2, 1),
                }),
                note_names: [
                    "C", "C♯", "D", "E♭", "E", "F", "F♯", "G", "A♭", "A", "B♭", "B",
                ]
                .into_iter()
                .map(str::to_string)
                .collect(),
            }],
            layout: vec![Layout {
                name: "5x3".to_string(),
                bbox: (1, 1, 8, 8),
                base: (2, 2),
                scale_name: "EDO-12".to_string(),
                steps: (2, 1),
                scale: None,
            }],
        };
        let c: ConfigFile = toml::from_str(CONFIG).unwrap();
        assert_eq!(c, exp);
    }
}
