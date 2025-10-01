use crate::parsing::diagnostics::code;
use crate::parsing::diagnostics::{Diagnostic, Diagnostics};
use crate::parsing::model::{
    Dynamic, DynamicChange, DynamicLeader, DynamicLine, Note, NoteLeader, NoteLine, RawDirective,
    RegularDynamic, ScaleBlock, Span, Spanned,
};
use num_rational::Ratio;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

mod directives;
use crate::pitch::Pitch;
pub use directives::*;

pub struct Score {
    _version: u32,
    pending_scale: Option<ScaleDefinition>,
    scales: HashMap<String, Arc<Scale>>,
    pending_score_block: Option<ScoreBlock>,
    score_blocks: Vec<ScoreBlock>,
    tunings: HashMap<String, Arc<Tuning>>, // empty string key is default tuning
    pending_dynamic_changes: HashMap<String, Spanned<RegularDynamic>>,
}
impl Score {
    pub fn take_pending_scale(&mut self) -> Option<ScaleDefinition> {
        self.pending_scale.take()
    }
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

pub struct Tuning {
    pub scale: Arc<Scale>,
    pub base_pitch: Pitch,
}

#[derive(Default)]
pub struct ScoreBlock {
    pub note_lines: Vec<NoteLine>,
    pub dynamic_lines: Vec<DynamicLine>,
}

static DEFAULT_SCALE: LazyLock<Arc<Scale>> = LazyLock::new(|| {
    let base_pitch = Pitch::must_parse("1");
    let mut pitches = Vec::new();
    let mut next_pitch = base_pitch.clone();
    let increment = Pitch::must_parse("^1|12");
    for _ in 0..=12 {
        pitches.push(next_pitch.clone());
        next_pitch *= &increment;
    }
    let notes = [
        ("c", pitches[0].clone()),
        ("c#", pitches[1].clone()),
        ("d%", pitches[1].clone()),
        ("d", pitches[2].clone()),
        ("d#", pitches[3].clone()),
        ("e%", pitches[3].clone()),
        ("e", pitches[4].clone()),
        ("e#", pitches[5].clone()),
        ("f%", pitches[4].clone()),
        ("f", pitches[5].clone()),
        ("f#", pitches[6].clone()),
        ("g%", pitches[6].clone()),
        ("g", pitches[7].clone()),
        ("g#", pitches[8].clone()),
        ("a%", pitches[8].clone()),
        ("a", pitches[9].clone()),
        ("a#", pitches[10].clone()),
        ("b%", pitches[10].clone()),
        ("b", pitches[11].clone()),
        ("b#", pitches[12].clone()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    Arc::new(Scale {
        definition: ScaleDefinition {
            span: (0..1).into(),
            name: "default".to_string(),
            base_pitch: base_pitch.clone(),
            cycle: Ratio::from_integer(2),
        },
        notes,
    })
});
static DEFAULT_TUNING: LazyLock<Arc<Tuning>> = LazyLock::new(|| {
    let scale = DEFAULT_SCALE.clone();
    let base_pitch = scale.definition.base_pitch.clone();
    Arc::new(Tuning { scale, base_pitch })
});

struct ScoreBlockValidator<'a> {
    score: &'a mut Score,
    diags: &'a Diagnostics,
    seen_note_lines: HashMap<(&'a String, u32), Span>,
    seen_dynamic_lines: HashMap<&'a String, Span>,
    note_line_bar_checks: Vec<Vec<(Ratio<u32>, Span)>>,
}

impl<'a> ScoreBlockValidator<'a> {
    fn new(score: &'a mut Score, diags: &'a Diagnostics) -> Self {
        Self {
            score,
            diags,
            seen_note_lines: Default::default(),
            seen_dynamic_lines: Default::default(),
            note_line_bar_checks: Vec::new(),
        }
    }

    fn check_duplicated_note_line(&mut self, leader: &'a Spanned<NoteLeader>) {
        let part = &leader.value.name.value;
        let note = leader.value.note.value;
        if let Some(old) = self.seen_note_lines.insert((part, note), leader.span) {
            self.diags.push(
                Diagnostic::new(
                    code::SCORE,
                    leader.span,
                    "a line for this part/note has already occurred in this block",
                )
                .with_context(old, "here is the previous occurrence"),
            )
        }
    }

    fn check_duplicated_dynamic_line(&mut self, leader: &'a Spanned<DynamicLeader>) {
        let part = &leader.value.name.value;
        if let Some(old) = self.seen_dynamic_lines.insert(part, leader.span) {
            self.diags.push(
                Diagnostic::new(
                    code::SCORE,
                    leader.span,
                    "a dynamic line for this part has already occurred in this block",
                )
                .with_context(old, "here is the previous occurrence"),
            )
        }
    }

    fn validate_note_line(&mut self, line: &'a NoteLine) {
        let mut bar_checks: Vec<(Ratio<u32>, Span)> = Vec::new();
        self.check_duplicated_note_line(&line.leader);
        let tuning = self.score.tuning_for_part(&line.leader.value.name.value);
        // Count up beats, track bar checks, and check note names.
        let mut prev_beats = Ratio::from_integer(1u32);
        let mut beats_so_far = Ratio::from_integer(0u32);
        let mut first = true;
        let mut last_note_span = line.leader.span;
        for note in &line.notes {
            last_note_span = note.span;
            let (is_bar_check, beats) = match &note.value {
                Note::Regular(r) => (false, r.duration),
                Note::Hold(h) => (false, h.duration),
                Note::BarCheck(_) => (true, None),
            };
            if first {
                if is_bar_check {
                    self.diags.err(
                        code::SCORE,
                        note.span,
                        "a line may not start with a bar check",
                    );
                } else if beats.is_none() {
                    self.diags.err(
                        code::SCORE,
                        note.span,
                        "the first note on a line must have an explicit duration",
                    );
                }
                first = false;
            }
            if is_bar_check {
                bar_checks.push((beats_so_far, note.span));
            } else {
                let beats = beats.map(|x| x.value).unwrap_or(prev_beats);
                prev_beats = beats;
                beats_so_far += beats;
            }
            if let Note::Regular(r_note) = &note.value {
                let name = &r_note.name.value;
                if !tuning.scale.notes.contains_key(name) {
                    self.diags.err(
                        code::SCORE,
                        note.span,
                        format!(
                            "note '{name}' is not in the current scale ('{}')",
                            tuning.scale.definition.name
                        ),
                    )
                }
            }
        }
        // Add a bar check for the whole line.
        let end_span = (last_note_span.end - 1..last_note_span.end).into();
        bar_checks.push((beats_so_far, end_span));
        self.note_line_bar_checks.push(bar_checks);
    }

    fn validate_bar_check_count(&self, sb: &'a ScoreBlock) -> Option<()> {
        // Make sure all the lines have the same number of bar checks.
        let mut bar_checks_okay = true;
        let mut last_num_bar_checks: Option<usize> = None;
        for lbc in &self.note_line_bar_checks {
            let num_bar_checks = lbc.len();
            if let Some(prev) = last_num_bar_checks
                && prev != num_bar_checks
            {
                bar_checks_okay = false;
                break;
            }
            last_num_bar_checks = Some(num_bar_checks);
        }
        if bar_checks_okay {
            return Some(());
        }
        let mut e = Diagnostic::new(
            code::SCORE,
            sb.note_lines[0].leader.span,
            "note lines in this score block have different numbers of bar checks",
        );
        for (i, v) in self.note_line_bar_checks.iter().enumerate() {
            e = e.with_context(
                sb.note_lines[i].leader.span,
                format!("this line has {}", v.len()),
            );
        }
        self.diags.push(e);
        None
    }

    fn validate_bar_check_consistency(&self, sb: &'a ScoreBlock) -> Option<()> {
        // All the note lines have the same number of bar checks. Make sure they all match.
        let num_bar_checks = self.note_line_bar_checks[0].len();
        let mut bar_checks_okay = true;
        for check_idx in 0..num_bar_checks {
            let mut different = false;
            let (exp, _span) = self.note_line_bar_checks[0][check_idx];
            for lbc in &self.note_line_bar_checks[1..] {
                let (actual, _span) = lbc[check_idx];
                if actual != exp {
                    different = true;
                }
            }
            if different {
                bar_checks_okay = false;
                let what = if check_idx + 1 == num_bar_checks {
                    "the total number of beats".to_string()
                } else {
                    format!("the number beats by bar check {}", check_idx + 1)
                };
                let mut e = Diagnostic::new(
                    code::SCORE,
                    sb.note_lines[0].leader.span,
                    format!("in this score block, {what} is inconsistent across lines",),
                );
                for lbc in &self.note_line_bar_checks {
                    let (this_one, span) = lbc[check_idx];
                    e = e.with_context(span, format!("beats up to here = {this_one}"));
                }
                self.diags.push(e);
            }
        }
        if bar_checks_okay { Some(()) } else { None }
    }

    fn validate_bar_checks(&self, sb: &'a ScoreBlock) -> Option<Vec<Ratio<u32>>> {
        // Check consistency of note line durations and bar checks.
        self.validate_bar_check_count(sb)?;
        self.validate_bar_check_consistency(sb)?;

        // Calculate the number of beats per "bar", where a bar is a group separated by bar
        // checks. If no bar checks, there is one bar containing the whole line. We can just
        // use the first line since we know all the lines are consistent and there is always at
        // least one line.
        let mut delta: Ratio<u32> = Ratio::from_integer(0);
        let mut beats_per_bar = Vec::new();
        for (total_beats, _) in &self.note_line_bar_checks[0] {
            beats_per_bar.push(*total_beats - delta);
            delta = *total_beats;
        }
        Some(beats_per_bar)
    }

    fn validate_dynamic_line(
        &mut self,
        line: &'a DynamicLine,
        beats_per_bar: &Option<Vec<Ratio<u32>>>,
    ) {
        self.check_duplicated_dynamic_line(&line.leader);
        let mut bar_check_idx = 0usize;
        let mut check_bars = beats_per_bar.is_some();
        let mut last_position: Option<Ratio<u32>> = None;
        let mut last_change: Option<Spanned<RegularDynamic>> = self
            .score
            .pending_dynamic_changes
            .remove(&line.leader.value.name.value);
        for dynamic in &line.dynamics {
            match &dynamic.value {
                Dynamic::Regular(r) => {
                    if check_bars
                        && let Some(beats_per_bar) = &beats_per_bar
                        && r.position.value > beats_per_bar[bar_check_idx]
                    {
                        self.diags.err(
                            code::SCORE,
                            r.position.span,
                            format!(
                                "this position exceeds the number of beats in this bar ({})",
                                beats_per_bar[bar_check_idx],
                            ),
                        );
                    }
                    if let Some(prev) = last_position
                        && r.position.value <= prev
                    {
                        self.diags.err(
                            code::SCORE,
                            r.position.span,
                            "this dynamic does not occur after the preceding one",
                        );
                    }
                    last_position = Some(r.position.value);
                    if let Some(last_change_ref) = last_change.as_ref() {
                        let last_level = last_change_ref.value.level;
                        match last_change_ref.value.change.unwrap().value {
                            DynamicChange::Crescendo => {
                                if r.level.value <= last_level.value {
                                    self.diags.push(
                                        Diagnostic::new(
                                            code::SCORE,
                                            r.level.span,
                                            "this dynamic level must be larger than the previous one, which contained a crescendo"
                                        ).with_context(last_change_ref.span, "here is the previous dynamic for this part")
                                    );
                                }
                            }
                            DynamicChange::Diminuendo => {
                                if r.level.value >= last_level.value {
                                    self.diags.push(
                                        Diagnostic::new(
                                            code::SCORE,
                                            r.level.span,
                                            "this dynamic level must be less than the previous one, which contained a diminuendo"
                                        ).with_context(last_change_ref.span, "here is the previous dynamic for this part")
                                    );
                                }
                            }
                        }
                    }
                    last_change = r.change.map(|_| Spanned::new(dynamic.span, r.clone()));
                }
                Dynamic::BarCheck(span) => {
                    last_position = None;
                    if check_bars && let Some(beats_per_bar_ref) = &beats_per_bar {
                        bar_check_idx += 1;
                        if bar_check_idx >= beats_per_bar_ref.len() {
                            self.diags.err(
                                code::SCORE,
                                *span,
                                format!(
                                    "too many bar checks; number expected: {}",
                                    beats_per_bar_ref.len() - 1,
                                ),
                            );
                            check_bars = false;
                        }
                    }
                }
            }
        }
        if let Some(last_change) = last_change {
            self.score
                .pending_dynamic_changes
                .insert(line.leader.value.name.value.clone(), last_change);
        }
        if let Some(beats_per_bar) = &beats_per_bar
            && bar_check_idx < beats_per_bar.len() - 1
        {
            self.diags.err(
                code::SCORE,
                line.leader.span,
                format!(
                    "not enough bar checks; number expected: {}",
                    beats_per_bar.len() - 1,
                ),
            );
        }
    }

    fn validate(&mut self, sb: &'a ScoreBlock) {
        for line in &sb.note_lines {
            self.validate_note_line(line);
        }
        let beats_per_bar = self.validate_bar_checks(sb);
        for line in &sb.dynamic_lines {
            self.validate_dynamic_line(line, &beats_per_bar);
        }
    }
}

impl Score {
    pub fn new(s: Syntoniq) -> Self {
        let scales = [("default".to_string(), DEFAULT_SCALE.clone())]
            .into_iter()
            .collect();
        Self {
            _version: s.version.value,
            pending_scale: None,
            scales,
            pending_score_block: None,
            score_blocks: Default::default(),
            tunings: Default::default(),
            pending_dynamic_changes: Default::default(),
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
                    span: x.name.span,
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
            Directive::Tune(x) => self.apply_tuning(diags, x),
            Directive::ResetTuning(x) => self.reset_tuning(x),
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
        let scale = Arc::new(Scale {
            definition,
            notes: name_to_pitch
                .into_iter()
                .map(|(name, (pitch, _))| (name, pitch))
                .collect(),
        });
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
        if sb.note_lines.is_empty() {
            // No point in doing anything
            diags.err(
                code::SCORE,
                sb.dynamic_lines[0].leader.span,
                "at least one note line is required in a score block",
            );
            return;
        }
        let mut v = ScoreBlockValidator::new(self, diags);
        v.validate(&sb);
        self.score_blocks.push(sb);
    }

    fn tuning_for_part(&mut self, part: &str) -> Arc<Tuning> {
        // Determine the name of the part we should use. If the part has a tuning, use it.
        // Otherwise, fall back to the empty string, which indicates the global tuning.
        let part_to_use = self.tunings.get(part).map(|_| part).unwrap_or("");
        // Get the tuning. If not defined, fall back to the default tuning.
        self.tunings
            .get(part_to_use)
            .cloned()
            .unwrap_or(DEFAULT_TUNING.clone())
    }

    fn apply_tuning(&mut self, diags: &Diagnostics, tuning: Tune) {
        let Some(scale) = self.scales.get(&tuning.scale.value).cloned() else {
            diags.err(
                code::TUNE,
                tuning.scale.span,
                format!("unknown scale '{}'", tuning.scale.value),
            );
            return;
        };
        // Look up tuning by part for each part we are trying to tune. If no part is specified,
        // this applies to the global tuning. The first part gathers the part names we care
        // about, and the second part gets the effective tuning for the part.
        let cur_tunings: HashMap<String, Arc<Tuning>> = if tuning.part.is_empty() {
            vec![""]
        } else {
            tuning.part.iter().map(|x| x.value.as_ref()).collect()
        }
        .into_iter()
        .map(|x| (x.to_string(), self.tuning_for_part(x)))
        .collect();

        // Get the base pitch for each part.
        let base_pitches: HashMap<String, Pitch> = if let Some(p) = &tuning.base_pitch {
            // Use this value for all parts, disregarding the current base pitch.
            cur_tunings
                .keys()
                .map(|part| (part.to_string(), p.value.clone()))
                .collect()
        } else if let Some(p) = &tuning.base_factor {
            // Multiply each  existing tuning's base pitch by the factor to get the new one.
            cur_tunings
                .iter()
                .map(|(part, existing)| (part.to_string(), &existing.base_pitch * &p.value))
                .collect()
        } else if let Some(n) = &tuning.base_note {
            // Make sure the note name is valid in voice
            let fall_back = &scale.definition.base_pitch;
            cur_tunings
                .iter()
                .map(|(part, existing)| {
                    let p = if let Some(p) = existing.scale.notes.get(&n.value) {
                        p.clone()
                    } else {
                        diags.err(
                            code::TUNE,
                            n.span,
                            format!(
                                "note '{}' is not present in scale '{}', which is the current scale for part '{}'",
                                n.value,
                                existing.scale.definition.name,
                                part,
                            ),
                        );
                        fall_back.clone()
                    };
                    (part.clone(), p)
                }).collect()
        } else {
            // Use the scale's default base pitch
            let p = scale.definition.base_pitch.clone();
            cur_tunings
                .keys()
                .map(|part| (part.to_string(), p.clone()))
                .collect()
        };
        // Create a tuning for each distinct base pitch with this scale. Then apply the tuning
        // to each specified part.
        let mut tunings_by_pitch = HashMap::<String, Arc<Tuning>>::new();
        for (part, base_pitch) in base_pitches {
            let tuning = tunings_by_pitch.entry(part.clone()).or_insert_with(|| {
                Arc::new(Tuning {
                    scale: scale.clone(),
                    base_pitch,
                })
            });
            self.tunings.insert(part, tuning.clone());
        }
    }

    fn reset_tuning(&mut self, reset_tuning: ResetTuning) {
        if reset_tuning.part.is_empty() {
            self.tunings.clear();
        } else {
            for p in reset_tuning.part {
                self.tunings.remove(&p.value);
            }
        }
    }

    pub fn do_final_checks(&mut self, diags: &Diagnostics) {
        for (part, dynamic) in &self.pending_dynamic_changes {
            diags.err(
                code::SCORE,
                dynamic.span,
                format!(
                    "for part '{part}', the last dynamic has an unresolved crescendo/diminuendo"
                ),
            );
        }
        if let Some(def) = &self.pending_scale {
            diags.err(
                code::SCALE,
                def.span,
                "this scale definition was incomplete at EOF",
            );
        }
    }
}
