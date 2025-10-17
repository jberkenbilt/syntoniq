use crate::parsing::model::{NoteBehavior, NoteOption, Span};
use crate::parsing::score::{Scale, Tuning};
use crate::pitch::Pitch;
use num_rational::Ratio;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(Serialize)]
pub struct Timeline<'s> {
    pub events: BTreeSet<Arc<TimelineEvent<'s>>>,
    pub scales: Vec<Arc<Scale<'s>>>,
    pub midi_instruments: BTreeMap<Cow<'s, str>, MidiInstrumentNumber>,
    pub csound_instruments: BTreeMap<Cow<'s, str>, CsoundInstrumentId<'s>>,
    /// Least common multiple of time denominators, useful for computing ticks per beat
    pub time_lcm: u32,
}

#[derive(Serialize, PartialOrd, PartialEq, Ord, Eq)]
pub struct TimelineEvent<'s> {
    pub time: Ratio<u32>,
    pub span: Span,
    pub data: TimelineData<'s>,
}

#[derive(Serialize, PartialOrd, PartialEq, Ord, Eq)]
pub enum TimelineData<'s> {
    // Keep these in the order in which they should appear in the timeline relative to other
    // events that happen at the same time.
    Tempo(TempoEvent),
    NoteOff(NoteOffEvent<'s>),
    Dynamic(DynamicEvent<'s>),
    NoteOn(NoteOnEvent<'s>),
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
pub struct NoteOnEvent<'s> {
    pub part: &'s str,
    pub note_number: u32,
    pub value: NoteValue<'s>,
}

#[derive(Serialize, PartialOrd, PartialEq, Ord, Eq)]
pub struct NoteOffEvent<'s> {
    pub part: &'s str,
    pub note_number: u32,
}

#[derive(Serialize, PartialOrd, PartialEq, Ord, Eq)]
pub struct NoteValue<'s> {
    pub text: &'s str,
    pub note_name: &'s str,
    pub tuning: Tuning<'s>,
    pub absolute_pitch: Pitch,
    /// Scale degrees from base pitch; add to 60 to get tuned MIDI note number
    pub absolute_scale_degree: i32,
    pub options: Vec<NoteOption>,
    pub behavior: Option<NoteBehavior>,
}

#[derive(Serialize, PartialOrd, PartialEq, Ord, Eq)]
pub struct DynamicEvent<'s> {
    pub text: &'s str,
    pub part: &'s str,
    pub start_level: u8,
    pub end_level: Option<WithTime<u8>>,
}

#[derive(Serialize, Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
pub struct MidiInstrumentNumber {
    pub bank: u16,
    pub instrument: u8,
}

#[derive(Serialize, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum CsoundInstrumentId<'s> {
    Number(u32),
    Name(Cow<'s, str>),
}
impl<'s> Display for CsoundInstrumentId<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CsoundInstrumentId::Number(n) => write!(f, "{n}"),
            CsoundInstrumentId::Name(s) => write!(f, "\"{s}\""),
        }
    }
}

#[derive(Serialize, PartialOrd, PartialEq, Ord, Eq)]
pub struct TempoEvent {
    pub bpm: Ratio<u32>,
    pub end_bpm: Option<WithTime<Ratio<u32>>>,
}
