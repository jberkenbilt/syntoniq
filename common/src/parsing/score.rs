use crate::parsing::diagnostics::code;
use crate::parsing::diagnostics::{Diagnostic, Diagnostics};
use crate::parsing::model::{
    Dynamic, DynamicChange, DynamicLeader, DynamicLine, Note, NoteBehavior, NoteLeader, NoteLine,
    RawDirective, RegularDynamic, ScaleBlock, Span, Spanned,
};
use num_rational::Ratio;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::mem;
use std::sync::{Arc, LazyLock};

mod directives;
use crate::parsing::{
    DynamicEvent, NoteOffEvent, NoteOnEvent, NoteValue, Timeline, TimelineData, TimelineEvent,
    WithTime,
};
use crate::pitch::Pitch;
pub use directives::*;

pub struct Score {
    _version: u32,
    pending_scale: Option<ScaleDefinition>,
    scales: HashMap<String, Arc<Scale>>,
    pending_score_block: Option<ScoreBlock>,
    score_blocks: Vec<ScoreBlock>,
    /// empty string key is default tuning
    tunings: HashMap<String, Arc<Tuning>>,
    pending_dynamic_changes: HashMap<String, WithTime<Spanned<RegularDynamic>>>,
    line_start_time: Ratio<u32>,
    timeline: Timeline,
}

#[derive(Serialize)]
pub struct ScaleDefinition {
    #[serde(skip)]
    pub span: Span,
    pub name: String,
    pub cycle: Ratio<u32>,
}

pub(crate) mod scale_notes {
    use crate::parsing::score::{NamedScaleDegree, ScaleDegree};
    use serde::Serialize;
    use serde::Serializer;
    use std::collections::HashMap;

    pub fn serialize<S: Serializer>(
        v: &HashMap<String, ScaleDegree>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        let mut notes: Vec<NamedScaleDegree> = v
            .iter()
            .map(|(name, degree)| NamedScaleDegree { name, degree })
            .collect();
        notes.sort_by_key(|x| (x.degree.degree, x.name));
        Vec::serialize(&notes, s)
    }
}

#[derive(Serialize)]
pub struct Scale {
    #[serde(flatten)]
    pub definition: ScaleDefinition,
    #[serde(with = "scale_notes")]
    pub notes: HashMap<String, ScaleDegree>,
    pub pitches: Vec<Pitch>,
}
#[derive(Serialize, Clone)]
pub struct ScaleDegree {
    pub base_relative: Pitch,
    pub degree: i32,
}
#[derive(Serialize)]
pub struct NamedScaleDegree<'a> {
    pub name: &'a String,
    #[serde(flatten)]
    pub degree: &'a ScaleDegree,
}

impl Scale {
    pub fn new(definition: ScaleDefinition, note_pitches: HashMap<String, Pitch>) -> Arc<Self> {
        // For each note, calculate its pitch relative to the base and normalized to within the
        // cycle. The results in a revised base-relative pitch and cycle offset. Sort the resulting
        // normalized base-relative pitches to determine scale degrees. It is normal for scales to
        // have notes that fall outside the cycle, such as B# in 12-TET, which has a base-relative
        // pitch of 2.

        // Gather notes based on normalized base pitch and cycle offset.
        struct Intermediate {
            name: String,
            orig_base: Pitch,
            normalized_base: Pitch,
            cycle_offset: i32,
        }

        let one_as_pitch = Pitch::from(Ratio::from_integer(1));
        let cycle_as_pitch = Pitch::from(definition.cycle);
        let inverted_cycle_as_pitch = cycle_as_pitch.invert();
        let mut intermediate: Vec<Intermediate> = Vec::new();
        let mut distinct_base_relative = HashSet::new();
        for (name, orig_base) in note_pitches {
            // This may not be the most efficient way to calculate this, but it's probably the
            // clearest. Calculate the cycle offset to normalize this to within a cycle.
            let mut normalized_base = orig_base.clone();
            let mut cycle_offset = 0;
            while normalized_base < one_as_pitch {
                normalized_base *= &cycle_as_pitch;
                cycle_offset -= 1;
            }
            while normalized_base >= cycle_as_pitch {
                normalized_base *= &inverted_cycle_as_pitch;
                cycle_offset += 1;
            }
            distinct_base_relative.insert(normalized_base.clone());
            intermediate.push(Intermediate {
                name,
                orig_base,
                normalized_base,
                cycle_offset,
            });
        }
        // Get a sorted list of distinct normalized base-relative pitches
        let mut pitches: Vec<Pitch> = distinct_base_relative.into_iter().collect();
        pitches.sort();
        // Map these to degree
        let degrees: HashMap<Pitch, i32> = pitches
            .iter()
            .enumerate()
            .map(|(i, pitch)| (pitch.clone(), i as i32))
            .collect();
        // Now we can compute the scale degree of each note
        let degrees_per_cycle = pitches.len() as i32;
        let notes = intermediate
            .into_iter()
            .map(|i| {
                let degree = degrees[&i.normalized_base];
                let s = ScaleDegree {
                    base_relative: i.orig_base,
                    degree: degree + (degrees_per_cycle * i.cycle_offset),
                };
                (i.name, s)
            })
            .collect();
        Arc::new(Self {
            definition,
            notes,
            pitches,
        })
    }
}

#[derive(Serialize, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Tuning {
    pub scale_name: String,
    pub base_pitch: Pitch,
}

#[derive(Default)]
pub struct ScoreBlock {
    pub note_lines: Vec<NoteLine>,
    pub dynamic_lines: Vec<DynamicLine>,
}

static DEFAULT_SCALE: LazyLock<Arc<Scale>> = LazyLock::new(|| {
    let start_pitch = Pitch::must_parse("1");
    let mut pitches = Vec::new();
    let mut next_pitch = start_pitch.clone();
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
    Scale::new(
        ScaleDefinition {
            span: (0..1).into(),
            name: "default".to_string(),
            cycle: Ratio::from_integer(2),
        },
        notes,
    )
});
static DEFAULT_TUNING: LazyLock<Arc<Tuning>> = LazyLock::new(|| {
    let scale = DEFAULT_SCALE.clone();
    let scale_name = scale.definition.name.clone();
    let base_pitch = Pitch::must_parse("220*^1|4");
    Arc::new(Tuning {
        scale_name,
        base_pitch,
    })
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
            let beats = if is_bar_check {
                bar_checks.push((beats_so_far, note.span));
                Ratio::from_integer(0)
            } else {
                let beats = beats.map(Spanned::value).unwrap_or(prev_beats);
                prev_beats = beats;
                self.score.update_time_lcm(beats);
                beats
            };
            if let Note::Regular(r_note) = &note.value
                && let Some(scale) = self.score.scales.get(&tuning.scale_name)
            {
                let name = &r_note.name.value;
                if let Some(scale_degree) = scale.notes.get(name).cloned() {
                    let time = beats_so_far + self.score.line_start_time;
                    let part = line.leader.value.name.value.clone();
                    let note_number = line.leader.value.note.value;
                    let cycle = r_note.octave.map(Spanned::value).unwrap_or(0);
                    let mut absolute_pitch = &tuning.base_pitch * &scale_degree.base_relative;
                    if cycle != 0 {
                        absolute_pitch *= &Pitch::from(scale.definition.cycle.pow(cycle as i32));
                    }
                    let absolute_scale_degree =
                        scale_degree.degree + (cycle as i32 * scale.pitches.len() as i32);
                    //TODO: There are currently no checks on `slide` behavior (e.g. that the last
                    // note isn't a slide) or representation of the slide duration. When we add a
                    // directive to configure that, we can come back to that issue.
                    let value = NoteValue {
                        note_name: name.clone(),
                        tuning: tuning.clone(),
                        absolute_pitch,
                        absolute_scale_degree,
                        options: r_note.options.iter().cloned().map(Spanned::value).collect(),
                        behavior: r_note.behavior.map(Spanned::value),
                    };
                    self.score.insert_event(
                        time,
                        note.span,
                        TimelineData::NoteOn(NoteOnEvent {
                            part: part.clone(),
                            note_number,
                            value,
                        }),
                    );
                    if let Some(behavior) = r_note.behavior
                        && behavior.value == NoteBehavior::Sustain
                    {
                        // Don't generate a note off event.
                    } else {
                        //TODO: consider how long we want a note to sound. For now, just sound
                        // for the entire note duration.
                        self.score.insert_event(
                            time + r_note.duration.map(Spanned::value).unwrap_or(prev_beats),
                            note.span,
                            TimelineData::NoteOff(NoteOffEvent {
                                part: part.clone(),
                                note_number,
                            }),
                        );
                    }
                } else {
                    self.diags.err(
                        code::SCORE,
                        note.span,
                        format!(
                            "note '{name}' is not in the current scale ('{}')",
                            tuning.scale_name
                        ),
                    )
                }
            }
            beats_so_far += beats;
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
        let mut last_change: Option<WithTime<Spanned<RegularDynamic>>> = self
            .score
            .pending_dynamic_changes
            .remove(&line.leader.value.name.value);
        let mut bar_start_time = self.score.line_start_time;
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
                    self.score.update_time_lcm(r.position.value);
                    if let Some(last_change_ref) = last_change.as_ref() {
                        let last_level = last_change_ref.item.value.level;
                        match last_change_ref.item.value.change.unwrap().value {
                            DynamicChange::Crescendo => {
                                if r.level.value <= last_level.value {
                                    self.diags.push(
                                        Diagnostic::new(
                                            code::SCORE,
                                            r.level.span,
                                            "this dynamic level must be larger than the previous one, which contained a crescendo",
                                        ).with_context(last_change_ref.item.span, "here is the previous dynamic for this part")
                                    );
                                }
                            }
                            DynamicChange::Diminuendo => {
                                if r.level.value >= last_level.value {
                                    self.diags.push(
                                        Diagnostic::new(
                                            code::SCORE,
                                            r.level.span,
                                            "this dynamic level must be less than the previous one, which contained a diminuendo",
                                        ).with_context(last_change_ref.item.span, "here is the previous dynamic for this part")
                                    );
                                }
                            }
                        }
                    }
                    let time = bar_start_time + r.position.value;
                    let part = line.leader.value.name.value.clone();
                    if let Some(ch) = last_change.take() {
                        // Push the event for the previously started dynamic change. This may also
                        // be the start of a new dynamic change or an instantaneous event.
                        self.score.insert_event(
                            ch.time,
                            ch.item.span,
                            TimelineData::Dynamic(DynamicEvent {
                                part: part.clone(),
                                start_level: ch.item.value.level.value,
                                end_level: Some(WithTime::new(time, r.level.value)),
                            }),
                        );
                    }
                    match r.change {
                        None => {
                            // This is an instantaneous event. It may also correspond with the end
                            // of the previous dynamic change event.
                            self.score.insert_event(
                                time,
                                dynamic.span,
                                TimelineData::Dynamic(DynamicEvent {
                                    part,
                                    start_level: r.level.value,
                                    end_level: None,
                                }),
                            );
                        }
                        Some(_) => {
                            // The event will be pushed when the dynamic change completes.
                            last_change =
                                Some(WithTime::new(time, Spanned::new(dynamic.span, r.clone())));
                        }
                    }
                }
                Dynamic::BarCheck(span) => {
                    last_position = None;
                    if check_bars && let Some(beats_per_bar_ref) = &beats_per_bar {
                        bar_start_time += beats_per_bar_ref[bar_check_idx];
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
        if let Some(x) = beats_per_bar {
            for beats in x {
                self.score.line_start_time += beats;
            }
        }
    }
}

impl Score {
    pub fn new(s: Syntoniq) -> Self {
        let default_scale = DEFAULT_SCALE.clone();
        let scales = [("default".to_string(), default_scale.clone())]
            .into_iter()
            .collect();
        let timeline = Timeline {
            scales: vec![default_scale],
            events: Default::default(),
            time_lcm: 1,
        };
        Self {
            _version: s.version.value,
            pending_scale: None,
            scales,
            pending_score_block: None,
            score_blocks: Default::default(),
            tunings: Default::default(),
            pending_dynamic_changes: Default::default(),
            line_start_time: Ratio::from_integer(0),
            timeline,
        }
    }

    pub fn into_timeline(self) -> Timeline {
        self.timeline
    }

    pub fn take_pending_scale(&mut self) -> Option<ScaleDefinition> {
        self.pending_scale.take()
    }

    fn insert_event(&mut self, time: Ratio<u32>, span: Span, data: TimelineData) {
        self.timeline
            .events
            .insert(Arc::new(TimelineEvent { time, span, data }));
    }

    fn update_time_lcm(&mut self, beats: Ratio<u32>) {
        let d = beats.denom();
        self.timeline.time_lcm = num_integer::lcm(self.timeline.time_lcm, *d);
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
                    cycle: x
                        .cycle_ratio
                        .map(Spanned::value)
                        .unwrap_or(Ratio::from_integer(2)),
                });
            }
            Directive::UseScale(x) => self.use_scale(diags, x),
            Directive::Transpose(x) => self.transpose(diags, x),
            Directive::SetBasePitch(x) => self.set_base_pitch(x),
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
        let scale = Scale::new(
            definition,
            name_to_pitch
                .into_iter()
                .map(|(name, (pitch, _))| (name, pitch))
                .collect(),
        );
        let span = scale.definition.span;
        if let Some(old) = self.scales.insert(name.clone(), scale.clone()) {
            diags.push(
                Diagnostic::new(
                    code::SCALE,
                    span,
                    format!("a scale called '{}' has already been defined", name),
                )
                .with_context(old.definition.span, "here is the previous definition"),
            );
        } else {
            self.timeline.scales.push(scale);
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

    fn cur_tunings(&mut self, part: &[Spanned<String>]) -> HashMap<String, Arc<Tuning>> {
        // Look up tuning by part for each part we are trying to tune. If no part is specified,
        // this applies to the global tuning. The first part gathers the part names we care
        // about, and the second part gets the effective tuning for the part.
        if part.is_empty() {
            vec![""]
        } else {
            part.iter().map(|x| x.value.as_ref()).collect()
        }
        .into_iter()
        .map(|x| (x.to_string(), self.tuning_for_part(x)))
        .collect()
    }

    fn use_scale(&mut self, diags: &Diagnostics, directive: UseScale) {
        let Some(scale) = self.scales.get(&directive.name.value).cloned() else {
            diags.err(
                code::TUNE,
                directive.name.span,
                format!("unknown scale '{}'", directive.name.value),
            );
            return;
        };
        let cur_tunings = self.cur_tunings(&directive.part);
        // Keep the same base pitch.
        let base_pitches: HashMap<String, Pitch> = cur_tunings
            .iter()
            .map(|(part, existing)| (part.to_string(), existing.base_pitch.clone()))
            .collect();
        self.apply_tuning(Some(&scale.definition.name), cur_tunings, base_pitches);
    }

    fn note_pitch_in_tuning(
        &self,
        diags: &Diagnostics,
        part: &str,
        tuning: &Tuning,
        note: &Spanned<String>,
    ) -> Pitch {
        if let Some(scale) = self.scales.get(&tuning.scale_name)
            && let Some(sd) = scale.notes.get(&note.value)
        {
            &sd.base_relative * &tuning.base_pitch
        } else {
            diags.err(
                code::TUNE,
                note.span,
                format!(
                    "note '{}' is not present in scale '{}', which is the current scale for part '{}'",
                    note.value,
                    tuning.scale_name,
                    part,
                ),
            );
            tuning.base_pitch.clone()
        }
    }

    fn transpose(&mut self, diags: &Diagnostics, directive: Transpose) {
        let cur_tunings = self.cur_tunings(&directive.part);
        // Get the base pitch for each part.
        let base_pitches: HashMap<String, Pitch> = {
            // Make sure the note name is valid in voice
            cur_tunings
                .iter()
                .map(|(part, existing)| {
                    let written =
                        self.note_pitch_in_tuning(diags, part, existing, &directive.written);
                    let from_pitch =
                        self.note_pitch_in_tuning(diags, part, existing, &directive.pitch_from);
                    let factor = &from_pitch / &written;
                    (part.clone(), &existing.base_pitch * &factor)
                })
                .collect()
        };
        self.apply_tuning(None, cur_tunings, base_pitches);
    }

    fn set_base_pitch(&mut self, directive: SetBasePitch) {
        let cur_tunings = self.cur_tunings(&directive.part);
        // Get the base pitch for each part.
        let base_pitches: HashMap<String, Pitch> = cur_tunings
            .iter()
            .map(|(part, existing)| {
                // Validate checked that exactly one of `absolute` or `relative` was defined.
                let p = directive
                    .absolute
                    .as_ref()
                    .map(|x| x.value.clone())
                    .unwrap_or_else(|| {
                        &existing.base_pitch * &directive.relative.as_ref().unwrap().value
                    });
                (part.to_string(), p)
            })
            .collect();
        self.apply_tuning(None, cur_tunings, base_pitches);
    }

    fn apply_tuning(
        &mut self,
        new_scale: Option<&str>,
        cur_tunings: HashMap<String, Arc<Tuning>>,
        base_pitches: HashMap<String, Pitch>,
    ) {
        // Create a tuning for each distinct base pitch with this scale. Then apply the tuning
        // to each specified part. It is known that cur_tunings and base_pitches have the same
        // keys.
        let mut tunings_by_pitch = HashMap::<Pitch, Arc<Tuning>>::new();
        let mut parts_by_pitch = HashMap::<Pitch, Vec<String>>::new();
        for (part, base_pitch) in base_pitches {
            let existing = cur_tunings[&part].as_ref();
            let tuning = tunings_by_pitch
                .entry(base_pitch.clone())
                .or_insert_with(|| {
                    Arc::new(Tuning {
                        scale_name: new_scale.unwrap_or(&existing.scale_name).to_string(),
                        base_pitch: base_pitch.clone(),
                    })
                });
            parts_by_pitch
                .entry(base_pitch)
                .or_default()
                .push(part.clone());
            self.tunings.insert(part, tuning.clone());
        }
    }

    fn reset_tuning(&mut self, reset_tuning: ResetTuning) {
        if reset_tuning.part.is_empty() {
            let mut old = HashMap::new();
            mem::swap(&mut old, &mut self.tunings);
        } else {
            let mut parts = Vec::new();
            for p in reset_tuning.part {
                self.tunings.remove(&p.value);
                parts.push(p.value.clone());
            }
        }
    }

    pub fn do_final_checks(&mut self, diags: &Diagnostics) {
        for (part, dynamic) in &self.pending_dynamic_changes {
            diags.err(
                code::SCORE,
                dynamic.item.span,
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
