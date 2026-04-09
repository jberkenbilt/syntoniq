use crate::parsing::model::Span;
use crate::parsing::score::{ScalesByName, serialize_scales};
use crate::pitch::Pitch;
use num_rational::Ratio;
use num_traits::ToPrimitive;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::{cmp, mem};

#[derive(Serialize)]
pub struct Timeline<'s> {
    pub events: BTreeSet<Arc<TimelineEvent<'s>>>,
    #[serde(with = "serialize_scales")]
    pub scales: Arc<ScalesByName<'s>>,
    pub midi_instruments: BTreeMap<Cow<'s, str>, MidiInstrumentNumber>,
    pub csound_instruments: BTreeMap<Cow<'s, str>, CsoundInstrumentId<'s>>,
    pub csound_global_instruments: Vec<CsoundGlobalInstrument<'s>>,
    pub csound_template: Option<Cow<'s, str>>,
    /// Least common multiple of time denominators, useful for computing ticks per beat. Callers
    /// should not count on 100% of denominators being a factor, but all denominators of note and
    /// duration values will be. This means you should take the numerator of the floor of the
    /// product of this and a time value if you need an integer.
    pub time_lcm: u32,
}

#[derive(Serialize)]
pub struct CsoundGlobalInstrument<'s> {
    pub instrument: CsoundInstrumentId<'s>,
    pub tail: Ratio<u32>,
}

pub struct TimeBoundaries {
    pub start_time: Ratio<u32>,
    pub end_time: Ratio<u32>,
}

enum TimePosition {
    Before,
    After,
    EndsAtStart,
    StartsAtEnd,
    Overlapping,
}

#[derive(Serialize, Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct TimelineEvent<'s> {
    pub time: Ratio<u32>,
    pub repeat_depth: usize,
    pub span: Span,
    pub data: TimelineData<'s>,
}
impl<'s> TimelineEvent<'s> {
    pub fn end_time(&self) -> Ratio<u32> {
        let maybe = match &self.data {
            TimelineData::Note(e) => e.value.pitches.last().map(|x| x.end_time),
            TimelineData::Tempo(e) => e.end_bpm.as_ref().map(|x| x.time),
            TimelineData::Dynamic(e) => e.end_level.as_ref().map(|x| x.time),
            TimelineData::Mark(_) | TimelineData::RepeatStart(_) | TimelineData::RepeatEnd(_) => {
                None
            }
        };
        maybe.unwrap_or(self.time)
    }

    fn add_or_subtract(v: &mut Ratio<u32>, delta: &Ratio<u32>, subtract: bool) {
        if subtract {
            if delta > v {
                *v = Ratio::from_integer(0);
            } else {
                *v -= delta;
            }
        } else {
            *v += delta;
        }
    }

    fn interpolate(
        value_start: &mut Ratio<u32>,
        value_end: &mut Ratio<u32>,
        event_start: &mut Ratio<u32>,
        event_end: &mut Ratio<u32>,
        boundaries: &TimeBoundaries,
    ) -> TimePosition {
        if *event_start > boundaries.end_time {
            // This event falls entirely after the end time. We don't care about it.
            TimePosition::After
        } else if *event_end == boundaries.start_time {
            // This event ends exactly at the boundary start time. How it is handled depends on
            // whether its effect lasts after it's finished, such as tempo.
            *value_start = *value_end;
            *event_start = *event_end;
            TimePosition::EndsAtStart
        } else if *event_start == boundaries.end_time {
            // This event starts exactly at the boundary start time. How it is handled depends on
            // whether we want to know about it, like RepeatEnd or Mark.
            *value_end = *value_start;
            *event_end = *event_start;
            TimePosition::StartsAtEnd
        } else if *event_end < boundaries.start_time {
            // This event falls entirely before the start time.
            TimePosition::Before
        } else {
            fn inner(
                start: Ratio<u32>,
                end: Ratio<u32>,
                duration: Ratio<u32>,
                offset: Ratio<u32>,
                value: &mut Ratio<u32>,
            ) {
                let elapsed_fraction = offset / duration;
                if start <= end {
                    let value_delta = end - start;
                    *value = start + elapsed_fraction * value_delta;
                } else {
                    let value_delta = start - end;
                    *value = start - elapsed_fraction * value_delta;
                }
            }
            let duration = *event_end - *event_start;
            let mut new_value_start = *value_start;
            let mut new_value_end = *value_end;
            let mut new_event_start = *event_start;
            let mut new_event_end = *event_end;
            if *event_start < boundaries.start_time {
                inner(
                    *value_start,
                    *value_end,
                    duration,
                    boundaries.start_time - *event_start,
                    &mut new_value_start,
                );
                new_event_start = boundaries.start_time;
            }
            if *event_end > boundaries.end_time {
                inner(
                    *value_start,
                    *value_end,
                    duration,
                    boundaries.end_time - *event_start,
                    &mut new_value_end,
                );
                new_event_end = boundaries.end_time;
            }
            *value_start = new_value_start;
            *value_end = new_value_end;
            *event_start = new_event_start;
            *event_end = new_event_end;
            TimePosition::Overlapping
        }
    }

    pub fn copy_with_time_delta(
        &self,
        delta: Ratio<u32>,
        boundaries: Option<&TimeBoundaries>,
        subtract: bool,
    ) -> Option<Self> {
        let mut data = self.data.clone();
        let mut event_start = self.time;
        if let Some(b) = boundaries {
            // Filter out or adjust an event based on where it falls within the region of time we
            // are interested in. At the end, if the event survives, the event start time is moved
            // forward to the time boundary.
            match &mut data {
                TimelineData::Tempo(e) => {
                    match &mut e.end_bpm {
                        // This is a gradual tempo change event. Interpolate to set the range
                        // based on where we are in the tempo change.
                        Some(end_bpm) => {
                            let time_pos = Self::interpolate(
                                &mut e.bpm,
                                &mut end_bpm.item,
                                &mut event_start,
                                &mut end_bpm.time,
                                b,
                            );
                            match time_pos {
                                TimePosition::Before | TimePosition::EndsAtStart => {
                                    // If the tempo change finished at or before the start time,
                                    // treat it as an instantaneous tempo event for the end BPM.
                                    e.bpm = end_bpm.item;
                                    e.end_bpm = None;
                                }
                                TimePosition::After | TimePosition::StartsAtEnd => return None,
                                TimePosition::Overlapping => {}
                            }
                        }
                        None => {
                            // Keep any instantaneous tempo event that happens any time before the
                            // end time.
                            if event_start >= b.end_time {
                                return None;
                            }
                        }
                    }
                }
                TimelineData::Dynamic(e) => {
                    if let Some(end_level) = &mut e.end_level {
                        // We are part way through a dynamic change. Interpolate and then
                        // force back to u8.
                        let mut start_value: Ratio<u32> = Ratio::from_integer(e.start_level.into());
                        let mut end_value: Ratio<u32> = Ratio::from_integer(end_level.item.into());
                        if !matches!(
                            Self::interpolate(
                                &mut start_value,
                                &mut end_value,
                                &mut event_start,
                                &mut end_level.time,
                                b,
                            ),
                            TimePosition::Overlapping
                        ) {
                            return None;
                        }
                        // Interpolated values can only ever move toward their midpoint, so
                        // rounding the resulted interpolated values will always result in
                        // values that are within the valid u8 range.
                        e.start_level = start_value.round().to_u8().unwrap();
                        end_level.item = end_value.round().to_u8().unwrap();
                    }
                }
                TimelineData::Note(e) => {
                    let pitches: Vec<PitchChange> = mem::take(&mut e.value.pitches);
                    for mut p in pitches {
                        // Interpolating pitch values is tricky, so use proxies to compute the
                        // argument to Pitch::interpolate.
                        let mut value_start = Ratio::from_integer(0);
                        let mut value_end = Ratio::from_integer(1);
                        if !matches!(
                            Self::interpolate(
                                &mut value_start,
                                &mut value_end,
                                &mut p.start_time,
                                &mut p.end_time,
                                b,
                            ),
                            TimePosition::Overlapping
                        ) {
                            continue;
                        }
                        if let Some(end_pitch) = &mut p.end_pitch {
                            let new_start_pitch =
                                Pitch::interpolate(&p.start_pitch, end_pitch, value_start);
                            let new_end_pitch =
                                Pitch::interpolate(&p.start_pitch, end_pitch, value_end);
                            p.start_pitch = new_start_pitch;
                            *end_pitch = new_end_pitch;
                        }
                        e.value.pitches.push(p);
                    }
                    if e.value.pitches.is_empty() {
                        return None;
                    }
                }
                TimelineData::Mark(_) => {
                    // Mark events are instantaneous, and we only care about them if they are in
                    // the time range, not at the boundaries.
                    if event_start <= b.start_time || event_start >= b.end_time {
                        return None;
                    }
                }
                TimelineData::RepeatStart(_) | TimelineData::RepeatEnd(_) => {
                    // These events are okay to keep when they appear at the boundaries.
                    if event_start < b.start_time || event_start > b.end_time {
                        return None;
                    }
                }
            }
            // Static or single-point-of-time events should be adjusted to be within range.
            // This will be harmlessly redundant for events that were interpolated if the
            // actual event start time was passed to interpolate, but that is not always the
            // case.
            event_start = cmp::max(event_start, b.start_time);
        }
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
                for p in e.value.pitches.iter_mut() {
                    Self::add_or_subtract(&mut p.start_time, &delta, subtract);
                    Self::add_or_subtract(&mut p.end_time, &delta, subtract);
                }
            }
            TimelineData::Mark(_) | TimelineData::RepeatStart(_) | TimelineData::RepeatEnd(_) => {}
        };

        let mut new_time = event_start;
        Self::add_or_subtract(&mut new_time, &delta, subtract);
        Some(Self {
            time: new_time,
            repeat_depth: self.repeat_depth,
            span: self.span,
            data,
        })
    }

    pub fn copy_for_repeat(&self, delta: Ratio<u32>) -> Self {
        // copy_with_time_delta always returns Some when time_boundaries is None.
        let mut event = self.copy_with_time_delta(delta, None, false).unwrap();
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
    pub velocity: u8,
    pub pitches: Vec<PitchChange<'s>>,
}

#[derive(Serialize, Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct PitchChange<'s> {
    pub text: &'s str,
    pub span: Span,
    pub start_pitch: Pitch,
    pub start_time: Ratio<u32>,
    pub end_pitch: Option<Pitch>,
    pub end_time: Ratio<u32>,
}

#[derive(Serialize, Debug, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub struct DynamicEvent<'s> {
    pub text: &'s str,
    pub part: &'s str,
    pub start_level: u8,
    pub end_level: Option<WithTime<u8>>,
}

#[derive(Default, Serialize, Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
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

    #[test]
    fn test_add_or_subtract() {
        // add_or_subtract is thoroughly tested through other means, but the way event filtering
        // is implemented prevents the boundary condition of subtracting too much from ever
        // happening organically.
        let mut v = Ratio::from_integer(3);
        TimelineEvent::add_or_subtract(&mut v, &Ratio::from_integer(1), true);
        assert_eq!(v, Ratio::from_integer(2));
        TimelineEvent::add_or_subtract(&mut v, &Ratio::from_integer(5), true);
        assert_eq!(v, Ratio::from_integer(0));
    }

    #[test]
    fn test_interpolate() {
        fn r(n: u32) -> Ratio<u32> {
            Ratio::from_integer(n)
        }
        fn tb(start_time: u32, end_time: u32) -> TimeBoundaries {
            TimeBoundaries {
                start_time: r(start_time),
                end_time: r(end_time),
            }
        }

        // These need to be mut. The allow statements are working around a RustRover false positive.
        // as of RustRover 2026.1.
        // https://youtrack.jetbrains.com/issue/RUST-20121/false-positive-unused-mut-with-macrorules
        #[allow(unused_mut)]
        let mut value_start;
        #[allow(unused_mut)]
        let mut value_end;
        #[allow(unused_mut)]
        let mut event_start;
        #[allow(unused_mut)]
        let mut event_end;
        macro_rules! interpolate {
            ($v1:expr, $v2:expr, $e1:expr, $e2:expr, $b1:expr, $b2:expr) => {{
                value_start = r($v1);
                value_end = r($v2);
                event_start = r($e1);
                event_end = r($e2);
                TimelineEvent::interpolate(
                    &mut value_start,
                    &mut value_end,
                    &mut event_start,
                    &mut event_end,
                    &tb($b1, $b2),
                )
            }};
        }
        assert!(matches!(
            interpolate!(60, 120, 0, 12, 3, 15),
            TimePosition::Overlapping
        ));
        assert_eq!(event_start, r(3));
        assert_eq!(event_end, r(12));
        assert_eq!(value_start, r(75));
        assert_eq!(value_end, r(120));

        assert!(matches!(
            interpolate!(60, 120, 0, 12, 9, 15),
            TimePosition::Overlapping
        ));
        assert_eq!(event_start, r(9));
        assert_eq!(event_end, r(12));
        assert_eq!(value_start, r(105));
        assert_eq!(value_end, r(120));

        assert!(matches!(
            interpolate!(60, 120, 0, 12, 12, 15),
            TimePosition::EndsAtStart
        ));
        assert_eq!(event_start, r(12));
        assert_eq!(event_end, r(12));
        assert_eq!(value_start, r(120));
        assert_eq!(value_end, r(120));

        assert!(matches!(
            interpolate!(60, 120, 3, 12, 0, 3),
            TimePosition::StartsAtEnd
        ));
        assert_eq!(event_start, r(3));
        assert_eq!(event_end, r(3));
        assert_eq!(value_start, r(60));
        assert_eq!(value_end, r(60));

        assert!(matches!(
            interpolate!(60, 120, 0, 12, 14, 15),
            TimePosition::Before
        ));

        assert!(matches!(
            interpolate!(60, 120, 3, 12, 0, 2),
            TimePosition::After
        ));
    }
}
