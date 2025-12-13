use crate::parsing::model::Span;
use crate::parsing::score::{ScalesByName, Tuning, serialize_scales};
use crate::pitch::Pitch;
use num_rational::Ratio;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Serialize)]
pub struct Timeline<'s> {
    pub events: BTreeSet<Arc<TimelineEvent<'s>>>,
    #[serde(with = "serialize_scales")]
    pub scales: Arc<ScalesByName<'s>>,
    pub midi_instruments: BTreeMap<Cow<'s, str>, MidiInstrumentNumber>,
    pub csound_instruments: BTreeMap<Cow<'s, str>, CsoundInstrumentId<'s>>,
    /// Least common multiple of time denominators, useful for computing ticks per beat. Callers
    /// should not count on 100% of denominators being a factor, but all denominators of note and
    /// duration values will be. This means you should take the numerator of the floor of the
    /// product of this and a time value if you need an integer.
    pub time_lcm: u32,
}

#[derive(Serialize, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct TimelineEvent<'s> {
    pub time: Ratio<u32>,
    pub repeat_depth: usize,
    pub span: Span,
    pub data: TimelineData<'s>,
}
impl<'s> TimelineEvent<'s> {
    fn add_or_subtract(v: &mut Ratio<u32>, delta: &Ratio<u32>, subtract: bool) {
        if subtract {
            *v -= delta;
        } else {
            *v += delta;
        }
    }

    pub fn copy_with_time_delta(&self, delta: Ratio<u32>, subtract: bool) -> Self {
        let mut data = self.data.clone();
        match &mut data {
            TimelineData::Tempo(e) => {
                if let Some(x) = e.end_bpm.as_mut() {
                    Self::add_or_subtract(&mut x.time, &delta, subtract);
                }
            }
            TimelineData::Dynamic(e) => {
                if let Some(x) = e.end_level.as_mut() {
                    Self::add_or_subtract(&mut x.time, &delta, subtract);
                }
            }
            TimelineData::Note(e) => {
                Self::add_or_subtract(&mut e.value.adjusted_end_time, &delta, subtract);
                Self::add_or_subtract(&mut e.value.end_time, &delta, subtract);
            }
            TimelineData::Mark(_) | TimelineData::RepeatStart(_) | TimelineData::RepeatEnd(_) => {}
        };
        let mut new_time = self.time;
        Self::add_or_subtract(&mut new_time, &delta, subtract);
        Self {
            time: new_time,
            repeat_depth: self.repeat_depth,
            span: self.span,
            data,
        }
    }

    pub fn copy_for_repeat(&self, delta: Ratio<u32>) -> Self {
        let mut event = self.copy_with_time_delta(delta, false);
        event.repeat_depth += 1;
        event
    }
}

#[derive(Serialize, Debug, Clone, PartialOrd, PartialEq, Ord, Eq)]
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

#[derive(Serialize, Debug, Clone, PartialOrd, PartialEq, Ord, Eq)]
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

impl TempoEvent {
    pub(crate) fn adjust(&mut self, factor: Ratio<u32>) {
        self.bpm *= factor;
        if let Some(end) = self.end_bpm.as_mut() {
            end.item *= factor;
        }
    }
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
