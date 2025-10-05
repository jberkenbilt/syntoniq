use crate::parsing::model::{NoteBehavior, NoteOption, Span};
use crate::parsing::score::{Scale, Tuning};
use crate::pitch::Pitch;
use num_rational::Ratio;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct Timeline {
    pub events: Vec<TimelineEvent>,
    pub scales: Vec<Arc<Scale>>,
    /// Least common multiple of time denominators, useful for computing ticks per beat
    pub time_lcm: u32,
}

#[derive(Serialize, PartialOrd, PartialEq, Ord, Eq)]
pub struct TimelineEvent {
    pub time: Ratio<u32>,
    pub span: Span,
    pub data: TimelineData,
}

#[derive(Serialize, PartialOrd, PartialEq, Ord, Eq)]
pub enum TimelineData {
    // Keep these in the order in which they should appear in the timeline relative to other
    // events that happen at the same time.
    NoteOff(NoteEvent),
    Tuning(TuningEvent),
    Dynamic(DynamicEvent),
    NoteOn(NoteEvent),
}

#[derive(Serialize, PartialOrd, PartialEq, Ord, Eq)]
pub struct WithTime<T: Serialize> {
    pub time: Ratio<u32>,
    pub item: T,
}
impl<T: Serialize> WithTime<T> {
    pub fn new(time: Ratio<u32>, item: T) -> Self {
        Self { time, item }
    }
}

#[derive(Serialize, PartialOrd, PartialEq, Ord, Eq)]
pub struct TuningEvent {
    pub tuning: Arc<Tuning>,
    pub parts: Vec<String>,
}

#[derive(Serialize, PartialOrd, PartialEq, Ord, Eq)]
pub struct NoteEvent {
    pub part: String,
    pub note_number: u32,
    pub value: NoteValue,
}

#[derive(Serialize, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub struct NoteValue {
    pub note_name: String,
    pub scale_name: String,
    pub absolute_pitch: Pitch,
    /// Scale degrees from base pitch; add to 60 to get tuned MIDI note number
    pub absolute_scale_degree: i32,
    pub options: Vec<NoteOption>,
    pub behavior: Option<NoteBehavior>,
}

#[derive(Serialize, PartialOrd, PartialEq, Ord, Eq)]
pub struct DynamicEvent {
    pub part: String,
    pub start_level: u8,
    pub end_level: Option<WithTime<u8>>,
}
