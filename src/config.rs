use crate::layout::Layout;
use crate::pitch::Pitch;
use crate::scale::{Scale, ScaleType};
use anyhow::bail;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct ScaleConfig {
    pub name: String,
    #[serde(flatten)]
    pub scale_type: ScaleType,
    pub base_pitch: Pitch,
    pub note_names: Vec<String>,
}
impl From<ScaleConfig> for Scale {
    fn from(value: ScaleConfig) -> Self {
        Self {
            name: value.name,
            scale_type: value.scale_type,
            orig_base_pitch: value.base_pitch.clone(),
            base_pitch: value.base_pitch,
            note_names: value.note_names,
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct LayoutConfig {
    pub name: String,
    pub scale_name: String,
    /// row, column of base pitch (EDO only)
    pub base: Option<(i8, i8)>,
    /// horizontal, vertical steps (EDO only)
    pub steps: Option<(i8, i8)>,
}

#[derive(Deserialize, Debug, PartialEq)]
struct ConfigFile {
    scale: Vec<ScaleConfig>,
    layout: Vec<LayoutConfig>,
}

pub struct Config {
    pub layouts: Vec<Arc<RwLock<Layout>>>,
}

impl Config {
    pub fn load(file: &PathBuf) -> anyhow::Result<Self> {
        let data = fs::read(file)?;
        let c: ConfigFile = toml::from_slice(&data)?;
        let mut scales_by_name = HashMap::new();
        for scale_config in c.scale {
            let scale: Scale = scale_config.into();
            let name = scale.name.clone();
            scale.validate()?;
            if scales_by_name
                .insert(name.clone(), Arc::new(scale))
                .is_some()
            {
                bail!("duplicated scale {}", name);
            }
        }
        let mut layouts = Vec::new();
        for layout_config in c.layout.into_iter() {
            let Some(scale) = scales_by_name.get(&layout_config.scale_name) else {
                bail!(
                    "layout {}: no scale {}",
                    layout_config.name,
                    layout_config.scale_name
                );
            };
            if matches!(scale.scale_type, ScaleType::EqualDivision(_))
                && (layout_config.steps.is_none() || layout_config.base.is_none())
            {
                bail!(
                    "layout {}: steps and base must be specified for EDO scale",
                    layout_config.name
                );
            }
            let layout = Layout {
                name: layout_config.name,
                base: layout_config.base,
                scale: scale.as_ref().to_owned(),
                steps: layout_config.steps,
            };
            layouts.push(Arc::new(RwLock::new(layout)));
        }
        Ok(Config { layouts })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitch::{Factor, Pitch};
    use crate::scale::{EqualDivision, ScaleType};

    #[test]
    fn test_toml() {
        const CONFIG: &str = r#"
[[scale]]
name = "12-EDO"
type = "EqualDivision"
divisions = [12, 2, 1]
base_pitch = "220*^3|12" # middle C for A-440 12-TET scale
note_names = ["C", "C♯", "D", "E♭", "E", "F", "F♯", "G", "A♭", "A", "B♭", "B"]
[[layout]]
name = "5x3"
base = [2, 2]
scale_name = "12-EDO"
steps = [2, 1]
"#;
        let exp = ConfigFile {
            scale: vec![ScaleConfig {
                name: "12-EDO".to_string(),
                base_pitch: Pitch::new(vec![
                    Factor::new(220, 1, 1, 1).unwrap(),
                    Factor::new(2, 1, 3, 12).unwrap(),
                ]),
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
            layout: vec![LayoutConfig {
                name: "5x3".to_string(),
                scale_name: "12-EDO".to_string(),
                base: Some((2, 2)),
                steps: Some((2, 1)),
            }],
        };
        let c: ConfigFile = toml::from_str(CONFIG).unwrap();
        assert_eq!(c, exp);
    }
}
