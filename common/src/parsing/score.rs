use crate::parsing::diagnostics::code;
use crate::parsing::diagnostics::{Diagnostic, Diagnostics};
use crate::parsing::model::{DynamicLine, NoteLine, RawDirective, ScaleBlock, Span, Spanned};
use crate::parsing::score_helpers::FromRawDirective;
use num_rational::Ratio;
use std::collections::{HashMap, HashSet};

mod directives;
use crate::pitch::Pitch;
pub use directives::*;

pub struct Score {
    pub version: u32,
    pub pending_scale: Option<ScaleDefinition>,
    pub scales: HashMap<String, Scale>,
    pub pending_score_block: Option<ScoreBlock>,
    pub score_blocks: Vec<ScoreBlock>,
}

pub struct ScaleDefinition {
    pub span: Span,
    pub name: String,
    pub base_pitch: Pitch,
    pub cycle: Ratio<u32>,
}

pub struct Scale {
    pub definition: ScaleDefinition,
    pub notes: HashMap<String, Pitch>,
}

#[derive(Default)]
pub struct ScoreBlock {
    pub note_lines: Vec<NoteLine>,
    pub dynamic_lines: Vec<DynamicLine>,
}

impl Score {
    pub fn new(s: Syntoniq) -> Self {
        Self {
            version: s.version.value,
            pending_scale: None,
            scales: Default::default(),
            pending_score_block: None,
            score_blocks: Default::default(),
        }
    }

    pub fn handle_directive(&mut self, diags: &Diagnostics, d: &RawDirective) {
        let Some(directive) = Directive::from_raw(diags, d) else {
            return;
        };
        match directive {
            Directive::Syntoniq(_) => {
                diags.err(
                    code::INITIALIZATION,
                    d.name.span,
                    "Syntoniq is already initialized",
                );
            }
            Directive::DefineScale(x) => {
                self.pending_scale = Some(ScaleDefinition {
                    span: x.span,
                    name: x.name.value,
                    base_pitch: x
                        .base_pitch
                        .map(Spanned::value)
                        .unwrap_or_else(|| Pitch::must_parse("220*^1|4")),
                    cycle: x
                        .cycle_ratio
                        .map(Spanned::value)
                        .unwrap_or(Ratio::from_integer(2)),
                });
            }
        }
    }

    pub fn handle_scale_block(
        &mut self,
        diags: &Diagnostics,
        definition: Option<ScaleDefinition>,
        sb: &ScaleBlock,
    ) {
        let mut pitches = HashMap::new();
        let mut name_to_pitch = HashMap::new();
        for note in &sb.notes {
            let span = note.value.pitch.span;
            let pitch = note.value.pitch.value.as_pitch().clone();
            if let Some(old) = pitches.insert(pitch.clone(), span) {
                diags.push(
                    Diagnostic::new(code::SCALE, span, "another note has this pitch")
                        .with_context(old, "here is the previous pitch with the same value"),
                );
            }
            for note_name in &note.value.note_names {
                let name = note_name.value.clone();
                let span = note_name.span;
                if let Some((_, old)) = name_to_pitch.insert(name.clone(), (pitch.clone(), span)) {
                    diags.push(
                        Diagnostic::new(code::SCALE, span, "another note has this name")
                            .with_context(old, "here is the previous note with the same name"),
                    )
                }
            }
        }

        let Some(definition) = definition else {
            diags.err(
                code::USAGE,
                sb.span,
                "a scale block must be immediately preceded by a scale definition",
            );
            return;
        };
        let name = definition.name.clone();
        let scale = Scale {
            definition,
            notes: name_to_pitch
                .into_iter()
                .map(|(name, (pitch, _))| (name, pitch))
                .collect(),
        };
        let span = scale.definition.span;
        if let Some(old) = self.scales.insert(name.clone(), scale) {
            diags.push(
                Diagnostic::new(
                    code::SCALE,
                    span,
                    format!("a scale called '{}' has already been defined", name),
                )
                .with_context(old.definition.span, "here is the previous definition"),
            );
        }
    }

    fn current_score_block(&mut self) -> &mut ScoreBlock {
        if self.pending_score_block.is_none() {
            self.pending_score_block = Some(Default::default());
        }
        self.pending_score_block.as_mut().unwrap()
    }

    pub fn add_note_line(&mut self, line: NoteLine) {
        self.current_score_block().note_lines.push(line);
    }

    pub fn add_dynamic_line(&mut self, line: DynamicLine) {
        self.current_score_block().dynamic_lines.push(line);
    }

    pub fn handle_score_block(&mut self, diags: &Diagnostics) {
        let Some(sb) = self.pending_score_block.take() else {
            return;
        };
        let mut seen_note_lines = HashMap::new();
        let mut seen_dynamic_lines = HashMap::new();
        for line in &sb.note_lines {
            let part = &line.leader.value.name.value;
            let note = line.leader.value.note.value;
            if let Some(old) = seen_note_lines.insert((part, note), line.leader.span) {
                diags.push(
                    Diagnostic::new(
                        code::SCORE,
                        line.leader.span,
                        "a line for this note has already occurred in this block",
                    )
                    .with_context(old, "here is the previous occurrence"),
                )
            }
        }
        for line in &sb.dynamic_lines {
            let part = &line.leader.value.name.value;
            if let Some(old) = seen_dynamic_lines.insert(part, line.leader.span) {
                diags.push(
                    Diagnostic::new(
                        code::SCORE,
                        line.leader.span,
                        "a dynamic line for this part has already occurred in this block",
                    )
                    .with_context(old, "here is the previous occurrence"),
                )
            }
        }

        // TODO: validate score blocks

        self.score_blocks.push(sb);
    }
}
