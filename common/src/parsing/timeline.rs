use crate::parsing::model::Span;
use crate::parsing::score::{Scale, Tuning};
use crate::pitch::Pitch;
use num_rational::Ratio;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
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
    pub repeat_depth: usize,
    pub span: Span,
    pub data: TimelineData<'s>,
}
impl<'s> TimelineEvent<'s> {
    pub fn copy_for_repeat(&self, delta: Ratio<u32>) -> Arc<Self> {
        let mut data = self.data.clone();
        match &mut data {
            TimelineData::Tempo(e) => {
                if let Some(x) = e.end_bpm.as_mut() {
                    x.time += delta;
                }
            }
            TimelineData::Dynamic(e) => {
                if let Some(x) = e.end_level.as_mut() {
                    x.time += delta;
                }
            }
            TimelineData::Note(e) => {
                e.value.adjusted_end_time += delta;
                e.value.end_time += delta;
            }
            TimelineData::Mark(_) | TimelineData::RepeatStart(_) | TimelineData::RepeatEnd(_) => {}
        };
        Arc::new(Self {
            time: self.time + delta,
            repeat_depth: self.repeat_depth + 1,
            span: self.span,
            data,
        })
    }
}

#[derive(Serialize, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub enum TimelineData<'s> {
    // Keep these in the order in which they should appear in the timeline relative to other
    // events that happen at the same time and span. (Unusual.)
    Tempo(TempoEvent),
    Dynamic(DynamicEvent<'s>),
    Note(NoteEvent<'s>),
    Mark(MarkEvent<'s>),
    RepeatStart(MarkEvent<'s>),
    RepeatEnd(MarkEvent<'s>),
}

#[derive(Serialize, Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct WithTime<T: Serialize + Clone> {
    pub time: Ratio<u32>,
    pub item: T,
}
impl<T: Serialize + Clone> WithTime<T> {
    pub fn new(time: Ratio<u32>, item: T) -> Self {
        Self { time, item }
    }
}

#[derive(Serialize, Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct PartNote<'s> {
    pub part: &'s str,
    pub note_number: u32,
}

#[derive(Serialize, Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct NoteEvent<'s> {
    #[serde(flatten)]
    pub part_note: PartNote<'s>,
    pub value: NoteValue<'s>,
}

#[derive(Serialize, Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct NoteValue<'s> {
    pub text: &'s str,
    pub note_name: &'s str,
    pub tuning: Tuning<'s>,
    pub absolute_pitch: Pitch,
    /// Scale degrees from base pitch; add to 60 to get tuned MIDI note number
    pub absolute_scale_degree: i32,
    pub velocity: u8,
    pub end_time: Ratio<u32>,
    pub adjusted_end_time: Ratio<u32>,
}

#[derive(Serialize, Clone, PartialOrd, PartialEq, Ord, Eq)]
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
impl<'s> CsoundInstrumentId<'s> {
    pub fn output(&self, note: Option<String>) -> String {
        let note = note.map(|x| format!(".{x}")).unwrap_or_default();
        match self {
            CsoundInstrumentId::Number(n) => format!("{n}{note}"),
            CsoundInstrumentId::Name(s) => format!("\"{s}{note}\""),
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct TempoEvent {
    pub bpm: Ratio<u32>,
    pub end_bpm: Option<WithTime<Ratio<u32>>>,
}

#[derive(Serialize, Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct MarkEvent<'s> {
    pub label: Cow<'s, str>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::Bound::Included;

    #[test]
    fn btree_set_iterator_behavior() {
        // This test is to demonstrate how to use BTreeSet to copy a range of items.
        let mut s: BTreeSet<(i32, i32)> = Default::default();
        s.insert((2, 1));
        s.insert((1, 2));
        let mark1 = (3, 1);
        s.insert(mark1);
        s.insert((2, 2)); // before mark1
        s.insert((4, 1)); // after mark1
        let mark2 = (5, 1);
        s.insert(mark2);
        s.insert((4, 2)); // after mark1, before mark2
        s.insert((6, 2)); // after mark2
        let iter = s.range((Included(mark1), Included(mark2)));
        assert_eq!(
            iter.cloned().collect::<Vec<_>>(),
            [(3, 1), (4, 1), (4, 2), (5, 1)]
        );
    }
}
