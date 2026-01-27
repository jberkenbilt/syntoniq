use crate::parsing::diagnostics::code;
use crate::parsing::diagnostics::{Diagnostic, Diagnostics};
use crate::parsing::model::{
    Dynamic, DynamicChange, DynamicLeader, DynamicLine, LayoutItemType, Note, NoteLeader, NoteLine,
    NoteModifier, RawDirective, RegularDynamic, RegularNote, Span, Spanned,
};
use num_rational::Ratio;
use serde::Serialize;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::Bound::Excluded;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::atomic::AtomicI32;
use std::sync::{Arc, LazyLock, RwLock};
use std::{cmp, mem};

mod directives;
mod generator;
use crate::parsing::layout::{
    Coordinate, IsomorphicMapping, Layout, LayoutMapping, Layouts, ManualMapping, MappingDetails,
};
use crate::parsing::pass2::Pass2;
use crate::parsing::score::generator::NoteGenerator;
use crate::parsing::{
    CsoundInstrumentId, DynamicEvent, MarkEvent, MidiInstrumentNumber, NoteEvent, NoteValue,
    Options, PartNote, PitchChange, TempoEvent, Timeline, TimelineData, TimelineEvent, WithTime,
    pass2, score_helpers,
};
use crate::pitch::Pitch;
pub use directives::*;
use to_static_derive::ToStatic;

pub const BUILTIN_SCALES: &str = include_str!("built-in-scales.stq");

#[derive(Clone, PartialOrd, PartialEq, Eq, Hash)]
pub struct LayoutKey<'s> {
    pub layout: Cow<'s, str>,
    pub keyboard: Cow<'s, str>,
}

#[derive(Clone)]
struct PendingNote<'s> {
    event: WithTime<Spanned<NoteEvent<'s>>>,
    tied: bool,
}

pub struct Score<'s> {
    src: &'s str,
    _version: u32,
    scales: HashMap<Cow<'s, str>, RefCell<ScaleBuilder<'s>>>,
    pending_score_block: Option<ScoreBlock<'s>>,
    score_blocks: Vec<ScoreBlock<'s>>,
    /// empty string key is default tuning
    tunings: HashMap<Cow<'s, str>, Tuning<'s>>,
    pending_dynamic_changes: HashMap<&'s str, WithTime<Spanned<RegularDynamic>>>,
    pending_notes: HashMap<PartNote<'s>, PendingNote<'s>>,
    pending_tempo: Option<WithTime<Spanned<TempoEvent>>>,
    tempo_in_flight_until: Option<Spanned<Ratio<u32>>>,
    line_start_time: Ratio<u32>,
    midi_instruments: HashMap<Cow<'s, str>, Span>,
    csound_instruments: HashMap<Cow<'s, str>, Span>,
    known_parts: HashSet<Cow<'s, str>>,
    marks: HashMap<Cow<'s, str>, MarkData<'s>>,
    layouts: HashMap<LayoutKey<'s>, LayoutData<'s>>,
    mappings: HashMap<Cow<'s, str>, MappingData<'s>>,
    timeline: Timeline<'s>,
}

pub type ScalesByName<'s> = BTreeMap<Cow<'s, str>, Arc<Scale<'s>>>;
pub(crate) mod serialize_scales {
    use crate::parsing::score::Scale;
    use serde::Serialize;
    use serde::Serializer;
    use std::borrow::Cow;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    pub fn serialize<S: Serializer>(
        v: &Arc<BTreeMap<Cow<str>, Arc<Scale>>>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        let mut scales: Vec<&Scale> = v.values().map(Arc::as_ref).collect();
        scales.sort_by_key(|x| x.definition.span);
        scales.serialize(s)
    }
}

#[derive(Serialize)]
pub struct ScoreOutput<'s> {
    pub timeline: Timeline<'s>,
    pub layouts: Layouts<'s>,
}

pub struct LayoutData<'s> {
    span: Span,
    layout: Arc<RwLock<Layout<'s>>>,
}

pub struct MappingData<'s> {
    span: Span,
    mapping: Arc<MappingDetails<'s>>,
    scale_name: Cow<'s, str>,
}

pub struct MarkData<'s> {
    event: Arc<TimelineEvent<'s>>,
    pending_dynamic_changes: HashMap<&'s str, WithTime<Spanned<RegularDynamic>>>,
    pending_notes: HashMap<PartNote<'s>, PendingNote<'s>>,
}

#[derive(Serialize, ToStatic)]
pub struct ScaleDefinition<'s> {
    #[serde(skip)]
    pub span: Span,
    pub name: Cow<'s, str>,
    pub cycle: Ratio<u32>,
}

pub(crate) mod scale_notes {
    use crate::parsing::score::{NamedScaleDegree, ScaleDegree};
    use serde::Serialize;
    use serde::Serializer;
    use std::borrow::Cow;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    pub fn serialize<S: Serializer>(
        v: &BTreeMap<Cow<str>, Arc<ScaleDegree>>,
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

#[derive(Serialize, ToStatic)]
pub struct Scale<'s> {
    #[serde(flatten)]
    pub definition: ScaleDefinition<'s>,
    #[serde(with = "scale_notes")]
    pub notes: BTreeMap<Cow<'s, str>, Arc<ScaleDegree>>,
    pub primary_names: Vec<Cow<'s, str>>,
    pub pitches: Vec<Pitch>,
}

#[derive(Default)]
pub struct Assignments {
    pub notes: HashMap<Cow<'static, str>, Pitch>,
    pub primary_names: HashMap<Pitch, Cow<'static, str>>,
}
impl Debug for Assignments {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut tuples: Vec<_> = self
            .notes
            .iter()
            .map(|(name, pitch)| (pitch, name))
            .collect();
        tuples.sort();
        for (pitch, name) in tuples {
            writeln!(f, "{name} -> {pitch}")?;
        }
        let mut tuples: Vec<_> = self.primary_names.iter().collect();
        tuples.sort();
        for (pitch, name) in tuples {
            writeln!(f, "{pitch} <- {name}")?;
        }
        write!(f, "---")
    }
}

/// Generate notes dynamically
pub trait Generator {
    /// If the name represents a pitch, the pitch.
    fn get_note(&self, diags: &Diagnostics, name: &Spanned<&str>) -> Option<Pitch>;
    fn assign_generated_notes(&self) -> Assignments;
}

pub struct ScaleBuilder<'s> {
    pub definition: ScaleDefinition<'s>,
    pub notes: HashMap<Cow<'s, str>, Pitch>,
    pub primary_names: HashMap<Pitch, Cow<'s, str>>,
    pub generator: Option<Box<dyn Generator>>,
}

#[derive(Serialize, Clone, ToStatic)]
pub struct ScaleDegree {
    /// Interval between pitch and scale base; may fall outside of cycle
    pub base_relative: Pitch,
    /// Normalized interval between pitch and scale base; falls within cycle
    pub normalized_relative: Pitch,
    /// Scale degree of base_relative; may extend outside of cycle
    pub degree: i32,
}

#[derive(Serialize)]
pub struct NamedScaleDegree<'s> {
    pub name: &'s str,
    #[serde(flatten)]
    pub degree: &'s ScaleDegree,
}

#[derive(Serialize, Clone, Debug, PartialOrd, PartialEq, ToStatic)]
/// This is created from a token in `define_manual_mapping`. It represents a note along with any
/// octave markers as it appears in the mapping definition.
pub struct MappingItem<'s> {
    /// Bare note name
    pub note_name: Cow<'s, str>,
    /// Cycles as present in the mapping
    pub cycle: i32,
    /// Pitch of the note relative to the base along with any octave markers
    pub adjusted_base_relative: Pitch,
}

impl<'s> ScaleBuilder<'s> {
    pub fn get_note(&mut self, diags: &Diagnostics, name: &Spanned<Cow<'s, str>>) -> Option<Pitch> {
        self.notes.get(&name.value).cloned().or_else(|| {
            let pitch = self.generator.as_ref()?.get_note(diags, &name.as_ref());
            if let Some(p) = &pitch {
                self.notes.insert(name.value.clone(), p.clone());
                self.primary_names
                    .entry(p.clone())
                    .or_insert(name.value.clone());
            }
            pitch
        })
    }

    pub fn into_scale(mut self) -> Scale<'s> {
        // For each note, calculate its pitch relative to the base and normalized to within the
        // cycle. The results in a revised base-relative pitch and cycle offset. Sort the resulting
        // normalized base-relative pitches to determine scale degrees. It is normal for scales to
        // have notes that fall outside the cycle, such as B# in 12-TET, which has a base-relative
        // pitch of 2.

        // Gather notes based on normalized base pitch and cycle offset.
        struct Intermediate<'s> {
            name: Cow<'s, str>,
            orig_relative: Pitch,
            normalized_relative: Pitch,
            cycle_offset: i32,
        }

        // Up to this point, for generated scales, the scale will only have notes the user actually
        // used in the score. If this scale divides an interval, we should fill in useful names for
        // the rest of the notes. This provides more useful data about the scale in the generated
        // JSON file, and for the keyboard program, it allows a more semantically meaningful note
        // name to be displayed. This code ensures that note names explicitly used by users will
        // always take precedence over computed note names, but every pitch in the scale will have
        // a name, regardless of whether it ever appeared in the score.
        let assignments = self
            .generator
            .map(|g| g.assign_generated_notes())
            .unwrap_or_default();
        for (name, pitch) in assignments.notes {
            self.notes.insert(name, pitch);
        }
        for (pitch, name) in assignments.primary_names.into_iter() {
            self.primary_names.entry(pitch).or_insert(name);
        }

        let mut intermediate: Vec<Intermediate> = Vec::new();
        let mut distinct_base_relative = HashSet::new();
        for (name, orig_relative) in self.notes {
            let (normalized_relative, cycle_offset) =
                orig_relative.normalized(self.definition.cycle);
            distinct_base_relative.insert(normalized_relative.clone());
            // Update the primary name map in case the only appearance of a pitch is not within
            // the cycle.
            let new_name = self
                .primary_names
                .get(&orig_relative)
                .unwrap_or(&name)
                .clone();
            self.primary_names
                .entry(normalized_relative.clone())
                .or_insert(new_name);
            intermediate.push(Intermediate {
                name,
                orig_relative,
                normalized_relative,
                cycle_offset,
            });
        }
        // Get a sorted list of distinct normalized base-relative pitches
        let mut pitches: Vec<Pitch> = distinct_base_relative.into_iter().collect();
        pitches.sort();
        // Map these to degree and primary note name.
        let degrees: HashMap<Pitch, i32> = pitches
            .iter()
            .enumerate()
            .map(|(i, pitch)| (pitch.clone(), i as i32))
            .collect();
        let primary_names: Vec<_> = pitches
            .iter()
            .map(|p| self.primary_names.get(p).cloned().unwrap())
            .collect();
        // Now we can compute the scale degree of each note
        let degrees_per_cycle = pitches.len() as i32;
        let notes: BTreeMap<_, _> = intermediate
            .into_iter()
            .map(|i| {
                let degree = degrees[&i.normalized_relative];
                let s = ScaleDegree {
                    base_relative: i.orig_relative,
                    normalized_relative: i.normalized_relative,
                    degree: degree + (degrees_per_cycle * i.cycle_offset),
                };
                (i.name, Arc::new(s))
            })
            .collect();

        Scale {
            definition: self.definition,
            notes,
            primary_names,
            pitches,
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Tuning<'s> {
    pub scale_name: Cow<'s, str>,
    pub base_pitch: Pitch,
}

#[derive(Default)]
pub struct ScoreBlock<'s> {
    pub note_lines: Vec<NoteLine<'s>>,
    pub dynamic_lines: Vec<DynamicLine<'s>>,
}

static DEFAULT_SCALE_NAME: &str = "12-EDO";
static DEFAULT_TUNING: LazyLock<Tuning<'static>> = LazyLock::new(|| {
    let base_pitch = Pitch::must_parse("220*^1|4");
    Tuning {
        scale_name: Cow::Borrowed(DEFAULT_SCALE_NAME),
        base_pitch,
    }
});

struct ScoreBlockValidator<'a, 's> {
    score: &'a mut Score<'s>,
    diags: &'a Diagnostics,
    seen_note_lines: HashMap<(&'s str, u32), Span>,
    seen_dynamic_lines: HashMap<&'s str, Span>,
    note_line_bar_checks: Vec<Vec<(Ratio<u32>, Span)>>,
}

impl<'a, 's> ScoreBlockValidator<'a, 's> {
    fn new(score: &'a mut Score<'s>, diags: &'a Diagnostics) -> Self {
        Self {
            score,
            diags,
            seen_note_lines: Default::default(),
            seen_dynamic_lines: Default::default(),
            note_line_bar_checks: Vec::new(),
        }
    }

    fn check_duplicated_note_line(&mut self, leader: &Spanned<NoteLeader<'s>>) {
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

    fn check_duplicated_dynamic_line(&mut self, leader: &Spanned<DynamicLeader<'s>>) {
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

    fn adjust_velocity_and_time(
        &mut self,
        r_note: &RegularNote<'s>,
        start_time: Ratio<u32>,
        value: &mut NoteValue<'s>,
    ) {
        let mut velocity: u8 = 72;
        let mut seen = HashSet::new();
        let tied = r_note.is_tie();
        let mut shorten: Ratio<u32> = Ratio::from_integer(0);
        for m in &r_note.modifiers {
            if !seen.insert(m.value) {
                if matches!(m.value, NoteModifier::Marcato | NoteModifier::Accent) {
                    self.diags
                        .err(code::SCORE, m.span, "accent marks may not be duplicated");
                }
                if matches!(m.value, NoteModifier::Tie | NoteModifier::Glide) {
                    self.diags
                        .err(code::SCORE, m.span, "tie and glide may not be duplicated");
                }
            }
            match m.value {
                NoteModifier::Accent => {
                    if seen.contains(&NoteModifier::Marcato) {
                        self.diags
                            .err(code::SCORE, m.span, "accent may not appear with marcato");
                    }
                    velocity = cmp::max(velocity, 96);
                }
                NoteModifier::Marcato => {
                    if seen.contains(&NoteModifier::Accent) {
                        self.diags
                            .err(code::SCORE, m.span, "marcato may not appear with accent");
                    }
                    velocity = cmp::max(velocity, 108);
                }
                NoteModifier::Shorten => {
                    // TODO: Make this amount configurable
                    shorten += Ratio::new(1, 4);
                    if tied {
                        self.diags.err(
                            code::SCORE,
                            m.span,
                            "shorten may not appear with a tied note",
                        );
                    }
                }
                NoteModifier::Tie | NoteModifier::Glide => {}
            }
        }
        value.velocity = velocity;
        if !tied && let Some(last_pitch) = value.pitches.last_mut() {
            let mut duration = last_pitch.end_time - start_time;
            let min_duration = cmp::min(duration, Ratio::new(1, 4));
            if duration - min_duration > shorten {
                duration -= shorten;
            } else {
                duration = min_duration;
            }
            last_pitch.end_time = start_time + duration;
        };
    }

    fn validate_note_line(&mut self, line: &NoteLine<'s>) {
        self.score
            .known_parts
            .insert(Cow::Borrowed(line.leader.value.name.value));
        let mut bar_checks: Vec<(Ratio<u32>, Span)> = Vec::new();
        self.check_duplicated_note_line(&line.leader);
        let tuning = self
            .score
            .tuning_for_part(&Cow::Borrowed(line.leader.value.name.value));
        // Count up beats, track bar checks, and check note names.
        let mut prev_beats = Ratio::from_integer(1u32);
        let mut beats_so_far = Ratio::from_integer(0u32);
        let mut first = true;
        let mut last_note_span = line.leader.span;
        let part = line.leader.value.name.value;
        let note_number = line.leader.value.note.value;
        let part_note = PartNote { part, note_number };
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
            let time = beats_so_far + self.score.line_start_time;
            match &note.value {
                Note::Regular(r_note) => {
                    let note_name = &r_note.note.name;
                    if let Some(scale) = self.score.scales.get(&tuning.scale_name)
                        && let Some(base_relative) =
                            { scale.borrow_mut().get_note(self.diags, note_name).clone() }
                    {
                        let cycle = r_note.note.octave.map(Spanned::value).unwrap_or(0);
                        let mut absolute_pitch = &tuning.base_pitch * &base_relative;
                        if cycle != 0 {
                            absolute_pitch *=
                                &Pitch::from(scale.borrow().definition.cycle.pow(cycle as i32));
                        }
                        let end_time =
                            time + r_note.duration.map(Spanned::value).unwrap_or(prev_beats);
                        // Get any note that might be currently sustained either by tie or glide.
                        let mut pending = self.score.pending_notes.remove(&part_note);
                        // If the current note is accented, end the pending note.
                        // If the pending note is a glide, set is end pitch.
                        if let Some(pending_note) = pending.as_mut() {
                            let pitches = &mut pending_note.event.item.value.value.pitches;
                            // There is guaranteed to be at least one pitch change.
                            let last_pitch = pitches.last_mut().unwrap();
                            if last_pitch.end_pitch.is_some() {
                                // A `Some` value is a place-holder for whatever the next pitch
                                // ends up being.
                                last_pitch.end_pitch = Some(absolute_pitch.clone());
                            }
                        }
                        let pending = pending.and_then(|pending_note| {
                            if !pending_note.tied
                                || r_note.modifiers.iter().any(|x| {
                                    matches!(x.value, NoteModifier::Accent | NoteModifier::Marcato)
                                })
                            {
                                self.score.insert_note(pending_note.event);
                                None
                            } else {
                                Some(pending_note)
                            }
                        });
                        let end_pitch = if r_note.is_glide() {
                            // Use a Some value as a placeholder. The pitch will be supplied
                            // when this is resolved.
                            Some(Pitch::unit())
                        } else {
                            None
                        };
                        let this_pitch = PitchChange {
                            text: &self.score.src[note.span],
                            span: note.span,
                            start_pitch: absolute_pitch,
                            start_time: time,
                            end_pitch,
                            end_time,
                        };
                        let mut pending_note = pending.unwrap_or_else(|| {
                            // There is no pending note, so make a new one.
                            let value = NoteValue {
                                text: &self.score.src[note.span],
                                velocity: 0,
                                pitches: Default::default(),
                            };
                            PendingNote {
                                event: WithTime::new(
                                    time,
                                    Spanned::new(note.span, NoteEvent { part_note, value }),
                                ),
                                tied: false, // conditionally set below
                            }
                        });
                        // Append this pitch change.
                        pending_note.event.item.value.value.pitches.push(this_pitch);
                        self.adjust_velocity_and_time(
                            r_note,
                            time,
                            &mut pending_note.event.item.value.value,
                        );
                        pending_note.tied = r_note.is_tie();
                        // If the current note is sustained, what we have is still pending. Otherwise,
                        // add it to the timeline.
                        if r_note.is_sustain() {
                            self.score.pending_notes.insert(part_note, pending_note);
                        } else {
                            self.score.insert_note(pending_note.event);
                        }
                    } else {
                        self.diags.err(
                            code::SCORE,
                            note.span,
                            format!(
                                "note '{}' is not in the current scale ('{}')",
                                note_name.value, tuning.scale_name,
                            ),
                        )
                    }
                }
                Note::Hold(h) => {
                    if let Some(p) = self.score.pending_notes.get_mut(&part_note) {
                        let end_time = time + h.duration.map(Spanned::value).unwrap_or(prev_beats);
                        // It is guaranteed that there is at least one pitch in any pending note.
                        // Extend the end time of the last pitch to cover the hold. For a tie, this
                        // extends the tie. For a glide, it extends the duration of the glide.
                        let pitches = &mut p.event.item.value.value.pitches;
                        let last_pitch = pitches.last_mut().unwrap();
                        last_pitch.end_time = end_time;
                    }
                }
                Note::BarCheck(_) => {}
            }
            beats_so_far += beats;
        }
        // Add a bar check for the whole line.
        let end_span = (last_note_span.end - 1..last_note_span.end).into();
        bar_checks.push((beats_so_far, end_span));
        self.note_line_bar_checks.push(bar_checks);
    }

    fn validate_bar_check_count(&self, sb: &ScoreBlock<'s>) -> Option<()> {
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

    fn validate_bar_check_consistency(&self, sb: &ScoreBlock<'s>) -> Option<()> {
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

    fn validate_bar_checks(&self, sb: &ScoreBlock<'s>) -> Option<Vec<Ratio<u32>>> {
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
        line: &DynamicLine<'s>,
        beats_per_bar: &Option<Vec<Ratio<u32>>>,
    ) {
        if !self
            .score
            .known_parts
            .contains(line.leader.value.name.value)
        {
            self.diags.err(
                code::SCALE,
                line.leader.value.name.span,
                format!("part {} is unknown", line.leader.value.name.value),
            );
        }
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
                    let part = line.leader.value.name.value;
                    if let Some(ch) = last_change.take() {
                        // Push the event for the previously started dynamic change. This may also
                        // be the start of a new dynamic change or an instantaneous event.
                        self.score.insert_event(
                            ch.time,
                            ch.item.span,
                            TimelineData::Dynamic(DynamicEvent {
                                text: &self.score.src[ch.item.span],
                                part,
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
                                    text: &self.score.src[dynamic.span],
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
                .insert(line.leader.value.name.value, last_change);
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

    fn validate(&mut self, sb: &ScoreBlock<'s>) {
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

impl<'s> Score<'s> {
    pub fn new(src: &'s str, s: Syntoniq) -> Self {
        let timeline = Timeline {
            scales: Default::default(),
            events: Default::default(),
            midi_instruments: Default::default(),
            csound_instruments: Default::default(),
            time_lcm: 1,
        };
        let pending_tempo = Some(WithTime::new(
            Ratio::from_integer(0),
            Spanned::new(
                0..1,
                TempoEvent {
                    bpm: Ratio::from_integer(72),
                    end_bpm: None,
                },
            ),
        ));
        let mut score = Self {
            src,
            _version: s.version.value,
            scales: Default::default(),
            pending_score_block: None,
            score_blocks: Default::default(),
            tunings: Default::default(),
            pending_dynamic_changes: Default::default(),
            pending_notes: Default::default(),
            pending_tempo,
            tempo_in_flight_until: None,
            line_start_time: Ratio::from_integer(0),
            midi_instruments: Default::default(),
            csound_instruments: Default::default(),
            known_parts: Default::default(),
            marks: Default::default(),
            layouts: Default::default(),
            mappings: Default::default(),
            timeline,
        };
        score.add_builtin_scales();
        score
    }

    fn add_builtin_scales(&mut self) {
        let tokens = pass2::parse2(BUILTIN_SCALES).unwrap();
        let temp_diags = Diagnostics::new();
        for tok in tokens {
            if let Pass2::Directive(rd) = tok.value.t {
                self.handle_directive(&temp_diags, Span::from(0..1), &rd);
            };
        }
        debug_assert!(!temp_diags.has_errors());
    }

    pub fn into_output(self) -> ScoreOutput<'s> {
        let mut layout_vec: Vec<(Span, Arc<RwLock<Layout>>)> = self
            .layouts
            .into_values()
            .map(|x| (x.span, x.layout))
            .collect();
        layout_vec.sort_by_key(|(span, _)| *span);
        let layouts = layout_vec
            .into_iter()
            .map(|(_, layout)| Arc::new(mem::take(&mut *layout.write().unwrap())))
            .collect();
        let scales: Arc<ScalesByName> = Arc::new(
            self.scales
                .into_iter()
                .map(|(name, scale)| (name, Arc::new(scale.into_inner().into_scale())))
                .collect(),
        );
        let mut timeline = self.timeline;
        timeline.scales = scales.clone();
        let layouts = Layouts { scales, layouts };
        ScoreOutput { timeline, layouts }
    }

    fn insert_event(&mut self, time: Ratio<u32>, span: Span, data: TimelineData<'s>) {
        // This is not the only way items get inserted into the timeline.
        self.timeline.events.insert(Arc::new(TimelineEvent {
            time,
            repeat_depth: 0,
            span,
            data,
        }));
    }

    fn insert_note(&mut self, note: WithTime<Spanned<NoteEvent<'s>>>) {
        self.insert_event(
            note.time,
            note.item.span,
            TimelineData::Note(note.item.value),
        );
    }

    fn update_time_lcm(&mut self, beats: Ratio<u32>) {
        let d = beats.denom();
        self.timeline.time_lcm = num_integer::lcm(self.timeline.time_lcm, *d);
    }

    pub fn handle_directive(&mut self, diags: &Diagnostics, span: Span, d: &RawDirective<'s>) {
        let Some(directive) = Directive::from_raw(diags, span, d) else {
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
            Directive::DefineScale(x) => self.define_scale(diags, x),
            Directive::DefineGeneratedScale(x) => self.define_generated_scale(diags, x),
            Directive::UseScale(x) => self.use_scale(diags, x),
            Directive::Transpose(x) => self.transpose(diags, x),
            Directive::SetBasePitch(x) => self.set_base_pitch(x),
            Directive::ResetTuning(x) => self.reset_tuning(x),
            Directive::MidiInstrument(x) => self.midi_instrument(diags, x),
            Directive::CsoundInstrument(x) => self.csound_instrument(diags, x),
            Directive::Tempo(x) => self.tempo(diags, x),
            Directive::Mark(x) => self.mark(diags, x),
            Directive::Repeat(x) => self.repeat(diags, x),
            Directive::DefineIsomorphicMapping(x) => self.define_isomorphic_mapping(diags, x),
            Directive::DefineManualMapping(x) => self.define_manual_mapping(diags, x),
            Directive::PlaceMapping(x) => self.place_mapping(diags, x),
        }
    }

    pub fn define_scale(&mut self, diags: &Diagnostics, directive: DefineScale<'s>) {
        let definition = ScaleDefinition {
            span: directive.scale.span,
            name: directive.scale.value,
            cycle: directive
                .cycle_ratio
                .map(Spanned::value)
                .unwrap_or(Ratio::from_integer(2)),
        };
        let scale_block = directive.scale_block.value;
        let mut pitches = HashMap::new();
        let mut name_to_pitch = HashMap::new();
        let mut pitch_to_name = HashMap::new();
        for note in &scale_block.notes.value {
            let span = note.value.pitch.span;
            let pitch = note.value.pitch.value.as_pitch().clone();
            if let Some(old) = pitches.insert(pitch.clone(), span) {
                diags.push(
                    Diagnostic::new(code::SCALE, span, "another note has this pitch")
                        .with_context(old, "here is the previous pitch with the same value"),
                );
            }
            for note_name in &note.value.note_names {
                let name = Cow::Borrowed(note_name.value);
                // Insert the first name encountered for a pitch.
                pitch_to_name.entry(pitch.clone()).or_insert(name.clone());
                let span = note_name.span;
                if let Some((_, old)) = name_to_pitch.insert(name, (pitch.clone(), span)) {
                    diags.push(
                        Diagnostic::new(code::SCALE, span, "another note has this name")
                            .with_context(old, "here is the previous note with the same name"),
                    )
                }
            }
        }
        let scale = ScaleBuilder {
            definition,
            notes: name_to_pitch
                .into_iter()
                .map(|(name, (pitch, _))| (name, pitch))
                .collect(),
            primary_names: pitch_to_name,
            generator: None,
        };
        self.add_scale(diags, scale);
    }

    fn add_scale(&mut self, diags: &Diagnostics, scale: ScaleBuilder<'s>) {
        let name = scale.definition.name.clone();
        let span = scale.definition.span;
        let scale = RefCell::new(scale);
        if let Some(old) = self.scales.insert(name.clone(), scale) {
            diags.push(
                Diagnostic::new(
                    code::SCALE,
                    span,
                    format!("a scale called '{}' has already been defined", name),
                )
                .with_context(
                    old.borrow().definition.span,
                    "here is the previous definition",
                ),
            );
        }
    }

    pub fn define_generated_scale(
        &mut self,
        diags: &Diagnostics,
        directive: DefineGeneratedScale<'s>,
    ) {
        let definition = ScaleDefinition {
            span: directive.scale.span,
            name: directive.scale.value,
            cycle: directive
                .cycle_ratio
                .map(Spanned::value)
                .unwrap_or(Ratio::from_integer(2)),
        };
        let divided_interval = directive
            .divided_interval
            .map(Spanned::value)
            .unwrap_or(definition.cycle);
        let generator: Option<Box<dyn Generator>> = Some(Box::new(NoteGenerator {
            divisions: directive.divisions.map(Spanned::value),
            divided_interval,
            tolerance: directive.tolerance.map(Spanned::value).unwrap_or_default(),
        }));
        let scale = ScaleBuilder {
            definition,
            notes: Default::default(),
            primary_names: Default::default(),
            generator,
        };
        self.add_scale(diags, scale);
    }

    fn current_score_block(&mut self) -> &mut ScoreBlock<'s> {
        if self.pending_score_block.is_none() {
            self.pending_score_block = Some(Default::default());
        }
        self.pending_score_block.as_mut().unwrap()
    }

    pub fn add_note_line(&mut self, line: NoteLine<'s>) {
        self.current_score_block().note_lines.push(line);
    }

    pub fn add_dynamic_line(&mut self, line: DynamicLine<'s>) {
        self.current_score_block().dynamic_lines.push(line);
    }

    fn handle_pending_tempo(&mut self, new_tempo: Option<WithTime<Spanned<TempoEvent>>>) {
        if let Some(t) = self.pending_tempo.take() {
            let insert_pending = match &new_tempo {
                None => true,
                Some(new) => new.time != t.time,
            };
            if insert_pending {
                self.insert_event(t.time, t.item.span, TimelineData::Tempo(t.item.value))
            }
        }
        self.pending_tempo = new_tempo;
    }

    pub fn handle_score_block(&mut self, diags: &Diagnostics) {
        let Some(sb) = self.pending_score_block.take() else {
            return;
        };
        self.handle_pending_tempo(None);
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

    fn tuning_for_part(&self, part: &Cow<'s, str>) -> Tuning<'s> {
        // Determine the name of the part we should use. If the part has a tuning, use it.
        // Otherwise, fall back to the empty string, which indicates the global tuning.
        let part_to_use = self
            .tunings
            .get(part)
            .map(|_| part)
            .unwrap_or(&Cow::Borrowed(""));
        // Get the tuning. If not defined, fall back to the default tuning.
        self.tunings
            .get(part_to_use)
            .cloned()
            .unwrap_or(DEFAULT_TUNING.clone())
    }

    fn cur_tunings(&mut self, part: &[Spanned<Cow<'s, str>>]) -> HashMap<Cow<'s, str>, Tuning<'s>> {
        // Look up tuning by part for each part we are trying to tune. If no part is specified,
        // this applies to the global tuning. The first part gathers the part names we care
        // about, and the second part gets the effective tuning for the part.
        if part.is_empty() {
            vec![Cow::Borrowed("")]
        } else {
            part.iter().map(|x| x.value.clone()).collect()
        }
        .into_iter()
        .map(|x| {
            let tuning = self.tuning_for_part(&x);
            (x, tuning)
        })
        .collect()
    }

    fn use_scale(&mut self, diags: &Diagnostics, directive: UseScale<'s>) {
        if !self.scales.contains_key(&directive.scale.value) {
            diags.err(
                code::TUNE,
                directive.scale.span,
                format!("unknown scale '{}'", directive.scale.value),
            );
            return;
        };
        let cur_tunings = self.cur_tunings(&directive.part);
        // Keep the same base pitch.
        let base_pitches: HashMap<Cow<'s, str>, Pitch> = cur_tunings
            .iter()
            .map(|(part, existing)| (part.clone(), existing.base_pitch.clone()))
            .collect();
        self.apply_tuning(Some(&directive.scale.value), cur_tunings, base_pitches);
    }

    fn note_pitch_in_tuning(
        &self,
        diags: &Diagnostics,
        part: &str,
        tuning: &Tuning<'s>,
        note: &Spanned<Cow<'s, str>>,
    ) -> Pitch {
        if let Some(scale) = self.scales.get(&tuning.scale_name)
            && let Some(base_relative) = { scale.borrow_mut().get_note(diags, note) }
        {
            &base_relative * &tuning.base_pitch
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

    fn transpose(&mut self, diags: &Diagnostics, directive: Transpose<'s>) {
        let cur_tunings = self.cur_tunings(&directive.part);
        // Get the base pitch for each part.
        let base_pitches: HashMap<Cow<'s, str>, Pitch> = {
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

    fn set_base_pitch(&mut self, directive: SetBasePitch<'s>) {
        let cur_tunings = self.cur_tunings(&directive.part);
        // Get the base pitch for each part.
        let base_pitches: HashMap<Cow<'s, str>, Pitch> = cur_tunings
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
                (part.clone(), p)
            })
            .collect();
        self.apply_tuning(None, cur_tunings, base_pitches);
    }

    fn apply_tuning(
        &mut self,
        new_scale: Option<&Cow<'s, str>>,
        cur_tunings: HashMap<Cow<'s, str>, Tuning<'s>>,
        base_pitches: HashMap<Cow<'s, str>, Pitch>,
    ) {
        // Create a tuning for each distinct base pitch with this scale. Then apply the tuning
        // to each specified part. It is known that cur_tunings and base_pitches have the same
        // keys.
        let mut tunings_by_pitch = HashMap::<Pitch, Tuning<'s>>::new();
        let mut parts_by_pitch = HashMap::<Pitch, Vec<Cow<'s, str>>>::new();
        for (part, base_pitch) in base_pitches {
            let existing = &cur_tunings[&part];
            let tuning = tunings_by_pitch
                .entry(base_pitch.clone())
                .or_insert_with(|| Tuning {
                    scale_name: new_scale.unwrap_or(&existing.scale_name).clone(),
                    base_pitch: base_pitch.clone(),
                });
            parts_by_pitch
                .entry(base_pitch)
                .or_default()
                .push(part.clone());
            self.tunings.insert(part, tuning.clone());
        }
    }

    fn reset_tuning(&mut self, reset_tuning: ResetTuning<'s>) {
        if reset_tuning.part.is_empty() {
            self.tunings.clear();
        } else {
            let mut parts = Vec::new();
            for p in reset_tuning.part {
                self.tunings.remove(&p.value);
                parts.push(p.value.clone());
            }
        }
    }

    fn midi_instrument(&mut self, diags: &Diagnostics, directive: MidiInstrument<'s>) {
        // Validate has checked ranges.
        let instrument = (directive.instrument.value - 1) as u8;
        let bank = directive.bank.map(|x| x.value - 1).unwrap_or(0) as u16;
        let midi_instrument = MidiInstrumentNumber { bank, instrument };
        score_helpers::check_duplicate_by_part(
            diags,
            "MIDI instrument",
            directive.part.as_slice(),
            directive.span,
            &mut self.midi_instruments,
            midi_instrument,
            &mut self.timeline.midi_instruments,
        );
    }

    fn csound_instrument(&mut self, diags: &Diagnostics, directive: CsoundInstrument<'s>) {
        // Validate has assured that exactly one of `name` or `number` is defined.
        let instrument = directive
            .name
            .map(|x| CsoundInstrumentId::Name(x.value))
            .unwrap_or_else(|| CsoundInstrumentId::Number(directive.number.unwrap().value));
        score_helpers::check_duplicate_by_part(
            diags,
            "Csound instrument",
            directive.part.as_slice(),
            directive.span,
            &mut self.csound_instruments,
            instrument,
            &mut self.timeline.csound_instruments,
        );
    }

    pub fn tempo(&mut self, diags: &Diagnostics, directive: Tempo<'s>) {
        let offset = directive
            .start_time
            .map(Spanned::value)
            .unwrap_or(Ratio::from_integer(0));
        let start_time = self.line_start_time + offset;
        if let Some(in_flight) = self.tempo_in_flight_until.as_ref() {
            if in_flight.value > start_time {
                let remaining = in_flight.value - start_time;
                diags.push(
                    Diagnostic::new(
                        code::SCORE,
                        directive.span,
                        format!(
                            "a tempo change is already in flight; beats until done: {remaining}"
                        ),
                    )
                    .with_context(in_flight.span, "here is the previous tempo directive"),
                );
            } else {
                self.tempo_in_flight_until.take();
            }
        }
        // Validate has verified that end_level and duration are both present or both absent.
        let end_bpm = directive.end_bpm.map(|level| {
            let end_time = directive.duration.unwrap().value + start_time;
            self.tempo_in_flight_until = Some(Spanned::new(directive.span, end_time));
            WithTime::new(end_time, level.value)
        });
        self.handle_pending_tempo(Some(WithTime::new(
            start_time,
            Spanned::new(
                directive.span,
                TempoEvent {
                    bpm: directive.bpm.value,
                    end_bpm,
                },
            ),
        )));
    }

    pub fn mark(&mut self, diags: &Diagnostics, directive: Mark<'s>) {
        let event = Arc::new(TimelineEvent {
            time: self.line_start_time,
            repeat_depth: 0,
            span: directive.label.span,
            data: TimelineData::Mark(MarkEvent {
                label: directive.label.value.clone(),
            }),
        });
        let mark_data = MarkData {
            event: event.clone(),
            pending_dynamic_changes: self.pending_dynamic_changes.clone(),
            pending_notes: self.pending_notes.clone(),
        };
        if let Some(old) = self.marks.insert(directive.label.value.clone(), mark_data) {
            diags.push(
                Diagnostic::new(
                    code::USAGE,
                    directive.label.span,
                    format!("mark '{}' has already appeared", directive.label.value),
                )
                .with_context(old.event.span, "here is the previous occurrence"),
            );
        }
        self.timeline.events.insert(event);
    }

    fn check_pending_over_repeat(
        diags: &Diagnostics,
        span: Span,
        pending_notes: &HashMap<PartNote<'s>, PendingNote<'s>>,
        pending_dynamic_changes: &HashMap<&'s str, WithTime<Spanned<RegularDynamic>>>,
    ) {
        if !pending_notes.is_empty() {
            let mut err =
                Diagnostic::new(code::SCORE, span, "notes may not be tied across repeats");
            for note in pending_notes.values() {
                err = err.with_context(note.event.item.span, "this sustain is unresolved");
            }
            diags.push(err);
        }
        if !pending_dynamic_changes.is_empty() {
            let mut err = Diagnostic::new(
                code::SCORE,
                span,
                "dynamic changes may not carry across repeats",
            );
            for note in pending_dynamic_changes.values() {
                err = err.with_context(note.item.span, "this dynamic change is unresolved");
            }
            diags.push(err);
        }
    }

    pub fn repeat(&mut self, diags: &Diagnostics, directive: Repeat<'s>) {
        let start = self.marks.get(&directive.start.value);
        let end = self.marks.get(&directive.end.value);
        for (mark, param) in [(start, &directive.start), (end, &directive.end)] {
            if mark.is_none() {
                diags.err(
                    code::SCORE,
                    param.span,
                    format!("mark '{}' is unknown", param.value),
                );
            }
        }
        let Some(start) = start else {
            return;
        };
        let Some(end) = end else {
            return;
        };
        if start.event.time >= end.event.time {
            diags.push(
                Diagnostic::new(
                    code::SCORE,
                    directive.span,
                    "for this repeat, the start mark does not precede the end mark",
                )
                .with_context(start.event.span, "here is the start")
                .with_context(end.event.span, "here is the end"),
            );
            // No point in further work at this point.
            return;
        }
        Self::check_pending_over_repeat(
            diags,
            directive.span,
            &end.pending_notes,
            &end.pending_dynamic_changes,
        );
        Self::check_pending_over_repeat(
            diags,
            directive.span,
            &self.pending_notes,
            &self.pending_dynamic_changes,
        );
        let start = start.event.clone();
        let end = end.event.clone();
        // Copy timeline events, adjusting the time.
        let duration = end.time - start.time;
        let delta = self.line_start_time - start.time;
        let end_time = self.line_start_time + duration;
        self.insert_event(
            self.line_start_time,
            directive.start.span,
            TimelineData::RepeatStart(MarkEvent {
                label: directive.start.value,
            }),
        );
        let to_copy: Vec<_> = self
            .timeline
            .events
            .range((Excluded(start), Excluded(end)))
            .cloned()
            .collect();
        for event in to_copy {
            let new_event = event.copy_for_repeat(delta);
            if let TimelineData::Tempo(tempo) = &new_event.data
                && let Some(end_bpm) = &tempo.end_bpm
                && end_bpm.time > end_time
            {
                // If this check is relaxed, it will have implications around mark/repeat. Study
                // the code in post_process carefully, and remember about Self::effective_tempo.
                // We would have to split tempo changes up around boundaries most likely. It's
                // probably better from an application design standpoint to not allow it as it would
                // be confusing for users in addition to being logically complex to code.
                let over_by = end_bpm.time - end_time;
                diags.push(Diagnostic::new(
                    code::SCORE,
                    directive.end.span,
                    "a tempo change started inside the repeated sections extends beyond the end of the section")
                    .with_context(
                        new_event.span,
                        format!("this tempo change exceeds the end of the repeated section; beats over: {over_by}"),
                    ));
            }
            self.timeline.events.insert(Arc::new(new_event));
        }
        self.insert_event(
            end_time,
            directive.end.span,
            TimelineData::RepeatEnd(MarkEvent {
                label: directive.end.value,
            }),
        );
        self.line_start_time += duration;
    }

    pub fn insert_mapping(
        &mut self,
        diags: &Diagnostics,
        name: Cow<'s, str>,
        mapping: MappingData<'s>,
    ) {
        let span = mapping.span;
        if let Some(old) = self.mappings.insert(name, mapping) {
            diags.push(
                Diagnostic::new(
                    code::DIRECTIVE_USAGE,
                    span,
                    "a mapping by this name has already been defined",
                )
                .with_context(old.span, "here is the previous definition"),
            );
        }
    }

    fn check_known_scale(
        &self,
        diags: &Diagnostics,
        scale_name: &Option<Spanned<Cow<'s, str>>>,
    ) -> Option<Cow<'s, str>> {
        let Some(scale) = scale_name else {
            return Some(Cow::Borrowed(DEFAULT_SCALE_NAME));
        };
        let r = self
            .scales
            .get(&scale.value)
            .map(|x| x.borrow().definition.name.clone());
        if r.is_none() {
            diags.err(
                code::DIRECTIVE_USAGE,
                scale.span,
                format!("scale '{}' is not known", scale.value),
            );
        }
        r
    }

    pub fn define_isomorphic_mapping(
        &mut self,
        diags: &Diagnostics,
        directive: DefineIsomorphicMapping<'s>,
    ) {
        let Some(scale_name) = self.check_known_scale(diags, &directive.scale) else {
            return;
        };
        let mapping = MappingDetails::Isomorphic(IsomorphicMapping {
            name: directive.mapping.value.clone(),
            steps_h: directive.steps_h.value as i32,
            steps_v: directive.steps_v.value as i32,
        });
        let mapping_data = MappingData {
            span: directive.mapping.span,
            mapping: Arc::new(mapping),
            scale_name,
        };
        self.insert_mapping(diags, directive.mapping.value, mapping_data);
    }

    pub fn define_manual_mapping(
        &mut self,
        diags: &Diagnostics,
        directive: DefineManualMapping<'s>,
    ) {
        let Some(scale_name) = self.check_known_scale(diags, &directive.scale) else {
            // Skip additional diagnostics if the scale is not known.
            return;
        };
        let mut anchor: Option<Spanned<Coordinate>> = None;
        let mut notes: Vec<Vec<Option<MappingItem>>> = Vec::new();
        let mut prev_row_len = 0usize;
        for (row_idx, row) in directive.layout_block.value.rows.value.iter().enumerate() {
            let mut note_row: Vec<Option<MappingItem>> = Vec::new();
            for (col_idx, item) in row.value.iter().enumerate() {
                if let Some(anchor_span) = item.value.is_anchor {
                    let anchor_coords = Coordinate {
                        row: row_idx as i32,
                        col: col_idx as i32,
                    };
                    if let Some(old_anchor) = anchor.take() {
                        diags.push(
                            Diagnostic::new(
                                code::LAYOUT,
                                anchor_span,
                                "a manual layout must have exactly one anchor",
                            )
                            .with_context(old_anchor.span, "here is the previous anchor"),
                        );
                    }
                    anchor = Some(Spanned::new(anchor_span, anchor_coords));
                }
                match &item.value.item {
                    LayoutItemType::Note(note) => {
                        let scale = self.scales.get(&scale_name).unwrap();
                        let sd = scale.borrow_mut().get_note(diags, &note.value.name);
                        match sd {
                            None => {
                                diags.err(
                                    code::LAYOUT,
                                    note.span,
                                    "this note is not in the scale for this mapping",
                                );
                                // Push something so counts are accurate.
                                note_row.push(None);
                            }
                            Some(note_base_relative) => {
                                let scale_ref = scale.borrow();
                                let cycle = note.value.octave.map(|x| x.value as i32).unwrap_or(0);
                                let adjusted_base_relative = &note_base_relative
                                    * &Pitch::from(scale_ref.definition.cycle.pow(cycle));
                                note_row.push(Some(MappingItem {
                                    note_name: note.value.name.value.clone(),
                                    cycle,
                                    adjusted_base_relative,
                                }));
                            }
                        };
                    }
                    LayoutItemType::Empty(_) => note_row.push(None),
                }
            }
            notes.push(note_row);
            let row_len = row.value.len();
            if row_idx >= 1 && row_len != prev_row_len {
                diags.err(
                    code::LAYOUT,
                    row.value[0].span,
                    format!(
                        "layout rows must be the same length; count for this row: {row_len}, previous row: {prev_row_len}"
                    ),
                )
            }
            prev_row_len = row_len;
        }
        let Some(mut anchor) = anchor else {
            diags.err(
                code::LAYOUT,
                directive.layout_block.span,
                "this layout has no anchor note; exactly one is required",
            );
            return;
        };
        // Rows appear in reverse order in the input since the lowest row is on the bottom.
        notes.reverse();
        anchor.value.row = notes.len() as i32 - 1 - anchor.value.row;
        let mapping = MappingDetails::Manual(ManualMapping {
            name: directive.mapping.value.clone(),
            h_factor: directive.h_factor.map(|x| x.value).unwrap_or_default(),
            v_factor: directive
                .v_factor
                .map(|x| x.value)
                .unwrap_or(Pitch::must_parse("2")),
            anchor_row: anchor.value.row,
            anchor_col: anchor.value.col,
            notes,
        });
        let mapping_data = MappingData {
            span: directive.mapping.span,
            mapping: Arc::new(mapping),
            scale_name,
        };
        self.insert_mapping(diags, directive.mapping.value, mapping_data);
    }

    pub fn place_mapping(&mut self, diags: &Diagnostics, directive: PlaceMapping<'s>) {
        let Some(mapping) = self.mappings.get(&directive.mapping.value) else {
            diags.err(
                code::LAYOUT,
                directive.mapping.span,
                format!("unknown mapping '{}'", directive.mapping.value),
            );
            return;
        };
        let base_pitch = directive
            .base_pitch
            .map(|x| x.value)
            .unwrap_or(self.tuning_for_part(&Cow::Borrowed("")).base_pitch.clone());
        let key = LayoutKey {
            layout: directive.layout.value.clone(),
            keyboard: directive.keyboard.value.clone(),
        };
        let layout = self.layouts.entry(key).or_insert_with(|| {
            LayoutData {
                span: directive.layout.span, // span of first reference to this layout
                layout: Arc::new(RwLock::new(Layout {
                    name: directive.layout.value,
                    keyboard: directive.keyboard.value,
                    mappings: vec![],
                    stagger: AtomicI32::new(0),
                })),
            }
        });
        let mut l = layout.layout.write().unwrap();
        l.mappings.push(LayoutMapping {
            name: mapping.mapping.name().clone(),
            scale: mapping.scale_name.clone(),
            base_pitch,
            anchor_row: directive.anchor_row.value as i32,
            anchor_col: directive.anchor_col.value as i32,
            rows_above: directive.rows_above.map(|x| x.value as i32),
            rows_below: directive.rows_below.map(|x| x.value as i32),
            cols_left: directive.cols_left.map(|x| x.value as i32),
            cols_right: directive.cols_right.map(|x| x.value as i32),
            details: mapping.mapping.clone(),
            offsets: Default::default(),
        })
    }

    pub fn do_final_checks(&mut self, diags: &Diagnostics) {
        let pending_notes = mem::take(&mut self.pending_notes);
        for pending in pending_notes.into_values() {
            diags.err(
                code::SCORE,
                pending.event.item.span,
                "this sustain was never resolved",
            );
        }
        for (part, &span) in &self.midi_instruments {
            if !part.is_empty() && !self.known_parts.contains(part) {
                diags.err(code::MIDI, span, "this part never appeared in the score");
            }
        }
        for (part, dynamic) in &self.pending_dynamic_changes {
            diags.err(
                code::SCORE,
                dynamic.item.span,
                format!(
                    "for part '{part}', the last dynamic has an unresolved crescendo/diminuendo"
                ),
            );
        }
    }

    fn effective_tempo(
        last_tempo_event: &TempoEvent,
        event_time: Ratio<u32>,
        current_time: Ratio<u32>,
    ) -> TempoEvent {
        let mut event = last_tempo_event.clone();
        if event_time > current_time {
            return event;
        }
        let Some(end_bpm) = &last_tempo_event.end_bpm else {
            // If no tempo change is in progress, the last tempo event is currently effective.
            return event;
        };
        if end_bpm.time <= current_time {
            // We have reached the tempo event's end time, so the effective tempo is the end tempo.
            event.bpm = end_bpm.item;
            event.end_bpm = None;
            return event;
        }
        // We are part way through a tempo change. Perform a linear interpolation to generate an
        // event that starts where we are now and ends where we would have ended.

        // Compute the tempo at this point.
        let tempo_delta = end_bpm.item - last_tempo_event.bpm;
        let tempo_duration = end_bpm.time - event_time;
        let elapsed_fraction = (current_time - event_time) / tempo_duration;
        let current = elapsed_fraction * tempo_delta + last_tempo_event.bpm;
        event.bpm = current;
        event
    }

    pub fn post_process(&mut self, diags: &Diagnostics, options: &Options) {
        for p in options.part.iter().map(Deref::deref) {
            if !self.known_parts.contains(&Cow::Borrowed(p)) {
                diags.err(
                    code::USAGE,
                    0..1,
                    format!("part '{p}', specified on the command line, is not known"),
                );
            }
        }
        if diags.has_errors() {
            return;
        }
        let tempo_percent = options.tempo_percent.unwrap_or(100);
        let tempo_factor = Ratio::new(tempo_percent, 100);
        // Filter timeline events
        let events = mem::take(&mut self.timeline.events);
        let events = if options.part.is_empty() {
            events
        } else {
            let parts: Vec<&str> = options.part.iter().map(Deref::deref).collect();
            events
                .into_iter()
                .filter(|event| {
                    let part = match &event.data {
                        TimelineData::Dynamic(e) => e.part,
                        TimelineData::Note(e) => e.part_note.part,
                        TimelineData::Tempo(_)
                        | TimelineData::Mark(_)
                        | TimelineData::RepeatStart(_)
                        | TimelineData::RepeatEnd(_) => {
                            return true;
                        }
                    };
                    let matches = parts.contains(&part);
                    if options.omit_parts {
                        !matches
                    } else {
                        matches
                    }
                })
                .collect()
        };
        let mut pending_tempo = None;
        let mut iter = events.into_iter();
        // Scan until we find the start mark. Keep track of the tempo. When scanning for marks,
        // we can always just match on the first occurrence. Marks that have a repeat depth > 1
        // will always appear after the original mark since repeat can only reference previously
        // seen marks.
        let mut delta: Ratio<u32> = Ratio::from_integer(0);
        if let Some(start_mark) = options.start_mark.as_ref() {
            let mut found_start = false;
            for event in iter.by_ref() {
                match &event.data {
                    TimelineData::Tempo(e) => pending_tempo = Some((event.clone(), e.clone())),
                    TimelineData::Mark(e) => {
                        if e.label.as_ref() == start_mark {
                            found_start = true;
                            delta = event.time;
                            break;
                        }
                        if let Some(end) = options.end_mark.as_ref()
                            && e.label.as_ref() == end
                        {
                            diags.err(code::USAGE, event.span, "end mark must preceded start mark");
                            return;
                        }
                    }
                    _ => {}
                }
            }
            if !found_start {
                diags.err(
                    code::USAGE,
                    0..1,
                    format!("requested start mark '{start_mark}' not found"),
                );
                return;
            }
        }
        if let Some(skip_beats) = options.skip_beats {
            delta += skip_beats;
        }
        let mut found_end = options.end_mark.is_none();
        let mut last_event_time = Ratio::from_integer(0);
        // Set into effect any tempo that would have been effective at this point in the timeline.
        let mut current_tempo = pending_tempo.map(|(timeline_event, tempo_event)| {
            let mut new_pending_tempo =
                Self::effective_tempo(&tempo_event, timeline_event.time, delta);
            new_pending_tempo.adjust(tempo_factor);
            let t = TimelineEvent {
                time: delta,
                repeat_depth: timeline_event.repeat_depth,
                span: timeline_event.span,
                data: TimelineData::Tempo(new_pending_tempo.clone()),
            };
            let new_event = t.copy_with_time_delta(delta, true);
            Arc::new(new_event)
        });
        for event in iter {
            let mut new_event = event.copy_with_time_delta(delta, true);
            if options.skip_repeats
                && (event.repeat_depth > 0 || matches!(event.data, TimelineData::RepeatEnd(_)))
            {
                // Skip repeated passages and advance delta so we don't have silence.
                delta += new_event.time - last_event_time;
                // Re-compute time with new delta.
                new_event = event.copy_with_time_delta(delta, true);
                if let TimelineData::Tempo(e) = &mut new_event.data {
                    // Keep track of tempo events inside skipped repeats, but don't insert
                    // anything. Regular validation logic ensures tempo changes that start inside
                    // repeated sections also finish within them, so we don't need in-flight
                    // computations.
                    e.adjust(tempo_factor);
                    current_tempo = Some(Arc::new(new_event));
                    continue;
                }
                continue;
            }
            match &mut new_event.data {
                TimelineData::Mark(e) => {
                    if let Some(end_mark) = options.end_mark.as_ref()
                        && end_mark == &e.label
                    {
                        found_end = true;
                        break;
                    }
                }
                TimelineData::RepeatStart(_) => {
                    last_event_time = new_event.time;
                    // Store the time of the repeat start, but don't put the event in the timeline.
                    if options.skip_repeats {
                        continue;
                    }
                }
                TimelineData::Note(note_event) => {
                    if note_event.value.pitches[0].start_time
                        == note_event.value.pitches.last().unwrap().end_time
                    {
                        // Skipped notes end up having zero duration. Omit from timeline.
                        continue;
                    }
                    if let Some(e) = current_tempo.take() {
                        self.timeline.events.insert(e);
                    }
                }
                TimelineData::Tempo(e) => {
                    e.adjust(tempo_factor);
                    if let Some(t) = current_tempo.take()
                        && t.time != new_event.time
                    {
                        self.timeline.events.insert(t);
                    }
                    let save = new_event.copy_with_time_delta(Ratio::from_integer(0), false);
                    current_tempo = Some(Arc::new(save));
                }
                TimelineData::Dynamic(_) | TimelineData::RepeatEnd(_) => {}
            }
            last_event_time = new_event.time;
            self.timeline.events.insert(Arc::new(new_event));
        }
        if !found_end {
            // found_end can only be false if end_mark is Some.
            diags.err(
                code::USAGE,
                0..1,
                format!(
                    "requested end mark '{}' not found",
                    options.end_mark.as_ref().unwrap()
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_tempo() {
        fn r(n: u32) -> Ratio<u32> {
            Ratio::from_integer(n)
        }

        let event = TempoEvent {
            bpm: r(60),
            end_bpm: Some(WithTime::new(r(12), r(120))),
        };
        assert_eq!(
            Score::effective_tempo(&event, r(6), r(3)),
            TempoEvent {
                bpm: r(60),
                end_bpm: event.end_bpm.clone(),
            }
        );
        assert_eq!(
            Score::effective_tempo(&event, r(6), r(9)),
            TempoEvent {
                bpm: r(90),
                end_bpm: event.end_bpm.clone(),
            }
        );
        assert_eq!(
            Score::effective_tempo(&event, r(6), r(12)),
            TempoEvent {
                bpm: r(120),
                end_bpm: None,
            }
        );
        assert_eq!(
            Score::effective_tempo(&event, r(6), r(15)),
            TempoEvent {
                bpm: r(120),
                end_bpm: None,
            }
        );
    }
}
