use anyhow::{anyhow, bail};
use midly::MetaMessage::{EndOfTrack, Tempo};
use midly::num::{u4, u7, u15, u24, u28};
use midly::{
    Arena, Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind,
};
use num_integer::Integer;
use num_rational::Ratio;
use std::cell::RefCell;
use std::cmp;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::Path;
use std::sync::Arc;
use syntoniq_common::parsing::model::NoteOption;
use syntoniq_common::parsing::score::{Scale, Tuning};
use syntoniq_common::parsing::{DynamicEvent, TempoEvent, Timeline, TimelineData, TimelineEvent};
use syntoniq_common::pitch::Pitch;

// Key concepts:
//   - A "part" is a syntoniq part, corresponding to a part in the score. A "port" is a MIDI port.
//     To reduce confusion, we will use `score_part` and `midi_port` rather than `part` and `port`.
//   - A tuning program consists of at most 128 notes. For the range of notes used in a current
//     scale, there is one tuning program for each 128 notes. A syntoniq tuning is a scale and
//     base pitch. A midi tuning is a subset of a syntoniq tuning.
//   - For simplicity and to allow sustaining notes from one tuning while playing notes from a
//     different tuning, we assign each channel a tuning.
//   - A track is a container for events. By convention, all events for a track are for the same
//     MIDI port and score part.
//
// Therefore:
//   - There is one MIDI port for every 15 channels (since we avoid channel 9 -- see comments)
//   - There is one (channel, MIDI port) pair for each (score part, tuning)
//   - There is one track for each (score part, MIDI port)
//
// We do the following up front:
//   - Assign a tuning program and, if needed, bank for each tuning.
//   - Dump all tunings to track 0 (midi track 1)
//   - Create a track for each (score part, MIDI port). At the beginning of the track,
//     set the track's instrument and MIDI port.
//   - For each (channel, MIDI port), get the (score part, tuning). In the first track for that
//     (score part, MIDI port), set the instrument and  tuning for that channel to the specific
//     tuning.
//
// Then, when we have a note:
//   - Use the syntoniq tuning and specific note to map to a tuning program.
//   - Use the tuning and score part to map to a channel and MIDI port.
//   - Use the score part and MIDI port to map to a track.
//   - Play the note in the track using the given channel.
//   - The note will have the correct instrument and MIDI port because of the track and the correct
//     tuning because of the channel.

#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Hash)]
struct NoteKey {
    part: String,
    note: u32,
}
struct PlayedNote {
    track: usize,
    channel: u4,
    key: u7,
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
struct PortChannel {
    midi_port: u7,
    channel: u4,
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
struct TrackPortChannel {
    track: usize,
    midi_port: u7,
    channel: u4,
}

struct MidiGenerator<'a> {
    arena: &'a Arena,
    timeline: &'a Timeline,
    scales_by_name: BTreeMap<&'a str, &'a Scale>,
    track_last_time: RefCell<BTreeMap<usize, u28>>,
    ticks_per_beat: u15,
    micros_per_beat: u24,
    last_played: BTreeMap<NoteKey, PlayedNote>,
    tuning_data: BTreeMap<&'a Tuning, Vec<TuningData>>,
    channel_data: BTreeMap<ChannelKey<'a>, PortChannel>,
    part_channels: BTreeMap<&'a str, BTreeSet<TrackPortChannel>>,
    track_data: BTreeMap<TrackKey<'a>, usize>,
    tracks: Vec<Vec<TrackEvent<'a>>>,
    smf: Option<Smf<'a>>,
}

#[derive(Debug)]
struct TuningData {
    range: Range<i32>,
    raw_program: i32,
    /// midi_offset + syntoniq note = midi note
    midi_offset: i32,
}
impl TuningData {
    fn new(range: Range<i32>, raw_program: i32) -> Self {
        // Opportunistically try to map in the normal MIDI way. Syntoniq 0 is MIDI note 60, so
        // MIDI range 0..128 corresponds to Syntoniq range -60..48.
        let midi_offset = if range.start >= -60 && range.end <= 68 {
            60
        } else {
            // Center these within 0..128.
            let first = ((128 - range.len()) / 2) as i32;
            first - range.start
        };
        Self {
            range,
            raw_program,
            midi_offset,
        }
    }

    /// Return tuning bank (if any) and program.
    fn tuning_program(raw_program: i32, use_banks: bool) -> anyhow::Result<(Option<u7>, u7)> {
        let bank = raw_program / 128;
        let prog = u7::try_from((raw_program % 128) as u8).unwrap();
        if use_banks {
            let bank = u8::try_from(bank + 1)
                .ok()
                .and_then(u7::try_from)
                .ok_or_else(|| anyhow!("maximum number of tunings exceeded"))?;
            Ok((Some(bank), prog))
        } else if bank > 0 {
            bail!("tuning_program called with use_banks = false and prog > 127");
        } else {
            Ok((None, prog))
        }
    }
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
struct ChannelKey<'a> {
    score_part: &'a str,
    raw_tuning: i32,
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
struct TrackKey<'a> {
    score_part: &'a str,
    midi_port: u7,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Eq, Ord)]
struct NoteRange {
    min_incl: i32,
    max_excl: i32,
}
impl NoteRange {
    fn widen(&mut self, scale_degree: i32) {
        self.min_incl = cmp::min(self.min_incl, scale_degree);
        self.max_excl = cmp::max(self.max_excl, scale_degree + 1);
    }
}

/// Given a string, return a byte vec that is exactly `size` bytes. This is done by truncating, if
/// necessary, at a character bounding, and then padding with space.
fn string_exact_bytes(s: &str, size: usize) -> Vec<u8> {
    // AI Generated.
    // Start from the desired cutoff, but not past the string.
    let mut end = size.min(s.len());
    // Move back to the previous UTF-8 boundary (at most 3 steps).
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    let mut out = vec![b' '; size];
    out[..end].copy_from_slice(&s.as_bytes()[..end]);
    out
}

/// Given a larger integer value, return (msb, lsb)
fn split_u14<T: TryInto<u16>>(val: T) -> anyhow::Result<(u7, u7)> {
    let as_u16 = val
        .try_into()
        .map_err(|_| anyhow!("range error mapping value to 16 bits"))?;
    let msb = u8::try_from(as_u16 / 128)
        .ok()
        .and_then(u7::try_from)
        .ok_or_else(|| anyhow!("range error getting msb of 14-bit value"))?;
    let lsb = u7::try_from((as_u16 % 128) as u8).unwrap();
    Ok((msb, lsb))
}

fn split_range(r: Range<i32>, n: usize) -> Vec<Range<i32>> {
    assert!(n > 0, "n must be > 0");
    let start = r.start as i64;
    let end = r.end as i64;
    if start >= end {
        return Vec::new();
    }
    let len = (end - start) as usize;
    let bins = len.div_ceil(n);
    let items_per_bin = len / bins;
    let mut rem = len % bins;

    let mut out = Vec::with_capacity(bins);
    let mut s = start;
    for _ in 0..bins {
        let size = items_per_bin
            + if rem > 0 {
                rem -= 1;
                1
            } else {
                0
            };
        let e = s + size as i64;
        out.push((s as i32)..(e as i32));
        s = e;
    }
    out
}

/// Ramp linearly from a start to an end level over the given number of ticks in at most `steps`
/// steps.
pub fn ramp(start_level: u8, end_level: u8, ticks: u32, steps: u32) -> Vec<(u32, u8)> {
    // AI generated with very specific prompt.
    if steps == 0 {
        return vec![(ticks, end_level)];
    }

    let s = start_level as i32;
    let e = end_level as i32;
    let d = e - s; // signed delta

    let mut out = Vec::new();
    let mut prev_level = s;

    for i in 1..=steps {
        // time uses plain floor for positive values
        let t = ((i as u128) * (ticks as u128) / (steps as u128)) as u32;

        // level uses Euclidean division to get mathematical floor for negatives
        let q = ((d as i128) * (i as i128)).div_euclid(steps as i128); // floor(d*i/steps)
        let level_i = s + q as i32;

        // Emit when the level changes, or always on the last step
        if level_i != prev_level || i == steps {
            // safe because level_i is between min(s,e) and max(s,e), within 0..=255
            let level_u8 = level_i.clamp(0, 255) as u8;
            out.push((t, level_u8));
            prev_level = level_i;
        }
    }
    out
}

fn ramp_rational(
    start_level: Ratio<u32>,
    end_level: Ratio<u32>,
    start_time: Ratio<u32>,
    duration: Ratio<u32>,
) -> Vec<(Ratio<u32> /*time*/, Ratio<u32> /*level*/)> {
    let steps: u32 = (duration * 4u32).ceil().to_integer();
    let mut result = Vec::with_capacity(steps as usize);

    for i in 1..=steps {
        let frac = Ratio::new(i, steps);
        let t = start_time + duration * frac;
        let v = start_level + (end_level - start_level) * frac;
        result.push((t, v));
    }

    result
}

fn bpm_to_micros_per_beat(bpm: Ratio<u32>) -> anyhow::Result<u24> {
    let &micros_per_beat = (Ratio::from_integer(60_000_000) / bpm).floor().numer();
    u24::try_from(micros_per_beat).ok_or_else(|| anyhow!("overflow calculating tempo"))
}

impl<'a> MidiGenerator<'a> {
    fn new(timeline: &'a Timeline, arena: &'a Arena) -> anyhow::Result<Self> {
        // Pick a timing that accommodates 2, 3, 5, and 7 as well as anything used by the score.
        let ticks_per_beat = u16::try_from(num_integer::lcm(timeline.time_lcm, 210))
            .ok()
            .and_then(u15::try_from)
            .ok_or_else(|| anyhow!("overflow calculating ticks per beat"))?;
        let micros_per_beat: u24 = 833333.into(); // 72 BPM -- changed by tempo events
        let scales_by_name = timeline
            .scales
            .iter()
            .map(|s| (s.definition.name.as_str(), s.as_ref()))
            .collect();
        Ok(Self {
            arena,
            timeline,
            scales_by_name,
            track_last_time: Default::default(),
            ticks_per_beat,
            micros_per_beat,
            last_played: Default::default(),
            tuning_data: Default::default(),
            channel_data: Default::default(),
            part_channels: Default::default(),
            track_data: Default::default(),
            tracks: Default::default(),
            smf: None,
        })
    }

    fn get_delta(&self, track: usize, event_time: Ratio<u32>) -> anyhow::Result<u28> {
        let time = u28::try_from(
            *(event_time * (u16::from(self.ticks_per_beat) as u32))
                .floor()
                .numer(),
        )
        .ok_or_else(|| anyhow!("time overflow"))?;
        let mut track_last_time = self.track_last_time.borrow_mut();
        let result = match track_last_time.entry(track) {
            Entry::Occupied(mut v) => {
                let last_time = v.get_mut();
                if time < *last_time {
                    bail!(
                        "time must be monotonically non-decreasing (track {track}, last={}, time={time})",
                        last_time
                    );
                }
                let result = time - *last_time;
                *last_time = time;
                result
            }
            Entry::Vacant(v) => {
                v.insert(time);
                time
            }
        };
        Ok(result)
    }

    fn tuning_for_note(&'a self, key: &'a Tuning, note: i32) -> anyhow::Result<&'a TuningData> {
        let data = self
            .tuning_data
            .get(key)
            .ok_or_else(|| anyhow!("no tuning data for known tuning"))?;
        data.iter()
            .find(|&i| i.range.contains(&note))
            .ok_or_else(|| anyhow!("internal error unable to get tuning data for note"))
    }

    fn get_all_tunings(&mut self) -> anyhow::Result<()> {
        // A given tuning may have up to 128 notes (midi note numbers 0 through 127). For each
        // scale, figure the range of notes used, and divide into tunings.
        let mut tunings: BTreeMap<&Tuning, NoteRange> = BTreeMap::new();
        for event in &self.timeline.events {
            let TimelineData::NoteOn(note_event) = &event.data else {
                continue;
            };
            let scale_degree = note_event.value.absolute_scale_degree;
            let entry = tunings
                .entry(&note_event.value.tuning)
                .or_insert(NoteRange {
                    min_incl: scale_degree,
                    max_excl: scale_degree + 1,
                });
            entry.widen(scale_degree);
        }
        // Divide tunings up so they contain no more than 128 notes. Then assign offsets and
        // channels to each tuning.
        let mut tunings: Vec<(&Tuning, NoteRange)> = tunings
            .into_iter()
            .filter(|(tuning, range)| {
                // Filter out the default MIDI tuning
                if tuning.scale_name == "default" && range.min_incl >= -60 && range.max_excl < 68 {
                    self.tuning_data
                        .insert(tuning, vec![TuningData::new(-60..68, 0)]);
                    false
                } else {
                    true
                }
            })
            .collect();
        tunings.sort();
        let mut program = 0;
        for (key, range) in tunings {
            let ranges = split_range(range.min_incl..range.max_excl, 128);
            let data = ranges
                .into_iter()
                .map(|range| {
                    program += 1;
                    TuningData::new(range, program)
                })
                .collect();
            self.tuning_data.insert(key, data);
        }
        Ok(())
    }

    fn get_channel_mappings(&mut self) -> anyhow::Result<()> {
        // Assign a separate channel to each (score_part, tuning) combination. Stay away from
        // channel 9 (from 0) since this is usually used for percussion. Here, we assign a "raw
        // channel", which is converted to a (midi_port, channel) pair.

        let mut channel_users: BTreeSet<ChannelKey> = BTreeSet::new();
        for event in &self.timeline.events {
            let TimelineData::NoteOn(note_event) = &event.data else {
                continue;
            };
            let tuning = self.tuning_for_note(
                &note_event.value.tuning,
                note_event.value.absolute_scale_degree,
            )?;
            let score_part = note_event.part.as_str();
            channel_users.insert(ChannelKey {
                score_part,
                raw_tuning: tuning.raw_program,
            });
        }
        let mut midi_port: u7 = 0.into();
        let mut channel: u4 = 0.into();
        let mut too_many = false;
        for channel_key in channel_users {
            if too_many {
                bail!("too many score part/tuning pairs");
            }
            self.channel_data
                .insert(channel_key, PortChannel { midi_port, channel });
            if channel == 8 {
                // Skip channel 9, percussion
                channel += 2.into();
            } else if channel == 15 {
                channel = 0.into();
                if midi_port == 127 {
                    too_many = true;
                } else {
                    midi_port += 1.into();
                }
            } else {
                channel += 1.into();
            }
        }
        Ok(())
    }

    fn use_banks(&self) -> bool {
        self.tuning_data.len() > 126
    }

    fn get_track_assignments(&mut self) -> anyhow::Result<()> {
        // A track should contain only notes for a single instrument/port. The first track
        // is for global information.
        let use_banks = self.use_banks();
        let mut track0 = Vec::new();
        if !self
            .timeline
            .events
            .iter()
            .any(|x| matches!(x.data, TimelineData::Tempo(_)))
        {
            // Insert a tempo event based on the default tempo.
            track0.push(TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(Tempo(self.micros_per_beat)),
            });
        }
        self.tracks.push(track0);
        let mut cur_track = 1usize;
        let mut channels_seen = BTreeSet::new();
        for (k, port_channel) in &self.channel_data {
            let track_key = TrackKey {
                score_part: k.score_part,
                midi_port: port_channel.midi_port,
            };
            if let Entry::Vacant(v) = self.track_data.entry(track_key) {
                v.insert(cur_track);
                cur_track += 1;
                self.tracks.push(vec![TrackEvent {
                    delta: 0.into(),
                    kind: TrackEventKind::Meta(MetaMessage::MidiPort(port_channel.midi_port)),
                }]);
            }
            let instrument = self
                .timeline
                .midi_instruments
                .get(k.score_part)
                .or_else(|| self.timeline.midi_instruments.get(""));
            if channels_seen.insert(port_channel) {
                // This is the first track for this port/channel, so assign the channel to its
                // fixed tuning program here.
                let track = self.tracks.last_mut().unwrap();
                select_tuning_program(track, port_channel.channel, k.raw_tuning, use_banks)?;
                if let Some(&instrument) = instrument {
                    if instrument.bank > 0 {
                        let (bank_msb, bank_lsb) = split_u14(instrument.bank)?;
                        track.push(TrackEvent {
                            delta: 0.into(),
                            kind: TrackEventKind::Midi {
                                channel: port_channel.channel,
                                message: MidiMessage::Controller {
                                    controller: 0.into(), // Bank Select MSB
                                    value: bank_msb,
                                },
                            },
                        });
                        track.push(TrackEvent {
                            delta: 0.into(),
                            kind: TrackEventKind::Midi {
                                channel: port_channel.channel,
                                message: MidiMessage::Controller {
                                    controller: 32.into(), // Bank Select LSB
                                    value: bank_lsb,
                                },
                            },
                        })
                    }
                    let program = u7::try_from(instrument.instrument)
                        .ok_or_else(|| anyhow!("overflow getting instrument number"))?;
                    track.push(TrackEvent {
                        delta: 0.into(),
                        kind: TrackEventKind::Midi {
                            channel: port_channel.channel,
                            message: MidiMessage::ProgramChange { program },
                        },
                    });
                }
            }
        }
        Ok(())
    }

    fn get_part_channels(&mut self) -> anyhow::Result<()> {
        // For each distinct part, make a list of all the tracks it uses. This is needed for
        // dynamics.
        for (channel_key, port_channel) in &self.channel_data {
            let track_key = TrackKey {
                score_part: channel_key.score_part,
                midi_port: port_channel.midi_port,
            };
            let &track = self.track_data.get(&track_key).ok_or_else(|| {
                anyhow!("get_part_channels: unable to get track for score_part/midi_port")
            })?;
            let tpc = TrackPortChannel {
                track,
                midi_port: port_channel.midi_port,
                channel: port_channel.channel,
            };
            self.part_channels
                .entry(channel_key.score_part)
                .or_default()
                .insert(tpc);
        }
        Ok(())
    }

    fn analyze(&mut self) -> anyhow::Result<()> {
        self.get_all_tunings()?;
        self.get_channel_mappings()?;
        self.get_track_assignments()?;
        self.get_part_channels()?;
        Ok(())
    }

    fn init_output(&mut self) -> anyhow::Result<()> {
        let header = Header::new(Format::Parallel, Timing::Metrical(self.ticks_per_beat));
        self.smf = Some(Smf::new(header));
        Ok(())
    }

    fn dump_tuning(
        &self,
        arena: &'a Arena,
        track: &mut Vec<TrackEvent<'a>>,
        tuning: &Tuning,
        data: &TuningData,
        use_banks: bool,
    ) -> anyhow::Result<()> {
        if data.raw_program == 0 {
            // We ensure program 0 is only assigned to the default 12-TET tuning. We don't need
            // to dump that.
            return Ok(());
        }
        let (bank, program) = TuningData::tuning_program(data.raw_program, use_banks)?;
        let mut dump: Vec<u8> = vec![
            0x7E, 0x7F, // all devices
        ];
        if let Some(bank) = bank {
            dump.append(&mut vec![
                0x08,
                0x04, // bulk bank dump reply
                bank.into(),
                program.into(),
            ]);
        } else {
            dump.append(&mut vec![
                0x08,
                0x01, // bulk dump reply
                program.into(),
            ]);
        }
        // Name: 16 bytes
        dump.append(&mut string_exact_bytes(&tuning.scale_name, 16));

        // This is the actual pitch calculation logic.
        let &scale = self
            .scales_by_name
            .get(tuning.scale_name.as_str())
            .ok_or_else(|| anyhow!("unknown scale in dump_tuning"))?;
        // Get basic information about the scale.
        let degrees = scale.pitches.len() as i32;
        let cycle_ratio = scale.definition.cycle;
        let base_pitch = &tuning.base_pitch;
        // Get the syntoniq degree for midi note 0.
        let first = -data.midi_offset;
        let (mut cycle, mut degree) = first.div_mod_floor(&degrees);
        let mut pitch0 = base_pitch.clone();
        let cycle_factor = Pitch::from(cycle_ratio);
        if cycle < 0 {
            let invert = cycle_factor.invert();
            while cycle < 0 {
                pitch0 *= &invert;
                cycle += 1;
            }
        }
        while cycle > 0 {
            pitch0 *= &cycle_factor;
            cycle -= 1;
        }
        for i in 0..128 {
            let pitch = &pitch0 * &scale.pitches[degree as usize];
            let mut v = pitch.midi_tuning().unwrap_or_else(|| {
                if i < data.midi_offset {
                    vec![0; 3]
                } else {
                    vec![127; 3]
                }
            });
            dump.append(&mut v);
            degree = (degree + 1) % degrees;
            if degree == 0 {
                pitch0 *= &cycle_factor;
            }
        }
        // Compute checksum per MTS spec. The MTS spec recommends that readers ignore the checksum.
        let mut checksum: u8 = 0;
        for b in &dump {
            checksum ^= b;
        }
        dump.push(checksum & 0x7F);
        // End SysEx
        dump.push(0x7F);
        let dump = arena.add(&dump);
        track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::SysEx(dump),
        });
        Ok(())
    }

    fn dump_tunings(&mut self) -> anyhow::Result<()> {
        let use_banks = self.use_banks();
        let mut tunings: Vec<_> = self.tuning_data.iter().collect();
        tunings.sort_by_key(|x| x.0);
        let mut track = Vec::new();
        for (&tuning, tuning_data_vec) in tunings {
            for tuning_data in tuning_data_vec {
                self.dump_tuning(self.arena, &mut track, tuning, tuning_data, use_banks)?;
            }
        }
        self.tracks[0].append(&mut track);
        Ok(())
    }

    fn turn_off_last_note(&mut self, last_on: &PlayedNote, delta: u28) {
        self.tracks[last_on.track].push(TrackEvent {
            delta,
            kind: TrackEventKind::Midi {
                channel: last_on.channel,
                message: MidiMessage::NoteOff {
                    key: last_on.key,
                    vel: 0.into(),
                },
            },
        });
    }

    fn volume_event(tpc: TrackPortChannel, delta: u28, value: u7) -> TrackEvent<'a> {
        TrackEvent {
            delta,
            kind: TrackEventKind::Midi {
                channel: tpc.channel,
                message: MidiMessage::Controller {
                    controller: 7.into(),
                    value,
                },
            },
        }
    }

    fn generate(mut self) -> anyhow::Result<Smf<'a>> {
        self.analyze()?;
        self.init_output()?;
        self.dump_tunings()?;

        let mut events: BTreeSet<_> = self.timeline.events.iter().cloned().collect();
        let mut last_event_time = events.first().unwrap().time;
        while let Some(event) = events.pop_first() {
            // We have to track last event time as we go since events may be inserted into the
            // even stream during processing.
            last_event_time = event.time;
            match &event.data {
                TimelineData::Tempo(e) => {
                    self.micros_per_beat = bpm_to_micros_per_beat(e.bpm)?;
                    // All tempo events go in track 0.
                    let delta = self.get_delta(0, event.time)?;
                    self.tracks[0].push(TrackEvent {
                        delta,
                        kind: TrackEventKind::Meta(Tempo(self.micros_per_beat)),
                    });
                    if let Some(t) = &e.end_bpm {
                        let end_bpm = t.item;
                        // The event comes with an absolute time. We need a duration.
                        let duration = t.time - event.time;
                        for (time, bpm) in ramp_rational(e.bpm, end_bpm, event.time, duration) {
                            events.insert(Arc::new(TimelineEvent {
                                time,
                                span: event.span,
                                data: TimelineData::Tempo(TempoEvent { bpm, end_bpm: None }),
                            }));
                        }
                    }
                }
                TimelineData::NoteOff(e) => {
                    let note_key = NoteKey {
                        part: e.part.clone(),
                        note: e.note_number,
                    };
                    if let Some(last_on) = self.last_played.remove(&note_key) {
                        let delta = self.get_delta(last_on.track, event.time)?;
                        self.turn_off_last_note(&last_on, delta);
                    } else {
                        eprintln!("TODO: warn about unexpected note")
                    }
                }
                TimelineData::Dynamic(e) => {
                    let part_channels = self
                        .part_channels
                        .get(e.part.as_str())
                        .ok_or_else(|| anyhow!("unable to get part channels"))?;
                    for &tpc in part_channels {
                        let delta = self.get_delta(tpc.track, event.time)?;
                        let value = u7::try_from(e.start_level)
                            .ok_or_else(|| anyhow!("volume out of range"))?;
                        self.tracks[tpc.track].push(Self::volume_event(tpc, delta, value));
                        if let Some(end_level) = &e.end_level {
                            let total_time = end_level.time - event.time;
                            let total_ticks = *(total_time * u16::from(self.ticks_per_beat) as u32)
                                .floor()
                                .numer();
                            let steps = 10;
                            for (ticks, level) in
                                ramp(e.start_level, end_level.item, total_ticks, steps)
                            {
                                let time =
                                    event.time + (Ratio::new(ticks, total_ticks) * total_time);
                                events.insert(Arc::new(TimelineEvent {
                                    time,
                                    span: event.span,
                                    data: TimelineData::Dynamic(DynamicEvent {
                                        part: e.part.clone(),
                                        start_level: level,
                                        end_level: None,
                                    }),
                                }));
                            }
                        }
                    }
                }
                TimelineData::NoteOn(e) => {
                    let note_key = NoteKey {
                        part: e.part.clone(),
                        note: e.note_number,
                    };
                    if let Some(last_on) = self.last_played.remove(&note_key) {
                        // Turn this note off. This would happen if the note were sustained.
                        let delta = self.get_delta(last_on.track, event.time)?;
                        self.turn_off_last_note(&last_on, delta);
                    }
                    let tuning_data =
                        self.tuning_for_note(&e.value.tuning, e.value.absolute_scale_degree)?;
                    let score_part = e.part.as_str();
                    let port_channel = self
                        .channel_data
                        .get(&ChannelKey {
                            score_part,
                            raw_tuning: tuning_data.raw_program,
                        })
                        .cloned()
                        .ok_or_else(|| anyhow!("unknown channel for note"))?;
                    let track_key = TrackKey {
                        score_part,
                        midi_port: port_channel.midi_port,
                    };
                    let track = self
                        .track_data
                        .get(&track_key)
                        .cloned()
                        .ok_or_else(|| anyhow!("unable to get track for note"))?;
                    let delta = self.get_delta(track, event.time)?;
                    let key = u8::try_from(e.value.absolute_scale_degree + tuning_data.midi_offset)
                        .ok()
                        .and_then(u7::try_from)
                        .ok_or_else(|| {
                            anyhow!("internal error: {}: note out of range", event.span)
                        })?;
                    self.last_played.insert(
                        note_key,
                        PlayedNote {
                            track,
                            channel: port_channel.channel,
                            key,
                        },
                    );
                    let mut vel: u7 = 72.into();
                    for o in &e.value.options {
                        match o {
                            NoteOption::Accent => vel = cmp::max(vel, 96.into()),
                            NoteOption::Marcato => vel = cmp::max(vel, 108.into()),
                        }
                    }
                    self.tracks[track].push(TrackEvent {
                        delta,
                        kind: TrackEventKind::Midi {
                            channel: port_channel.channel,
                            message: MidiMessage::NoteOn { key, vel },
                        },
                    })
                }
            }
        }
        let deltas: Vec<_> = (0..self.tracks.len())
            .map(|track| self.get_delta(track, last_event_time).unwrap())
            .collect();
        for (track_idx, track) in self.tracks.iter_mut().enumerate() {
            track.push(TrackEvent {
                delta: deltas[track_idx],
                kind: TrackEventKind::Meta(EndOfTrack),
            });
        }

        let mut smf = self.smf.take().unwrap();
        smf.tracks = self.tracks;
        Ok(smf)
    }
}

fn select_tuning_program(
    track: &mut Vec<TrackEvent>,
    channel: u4,
    raw_tuning: i32,
    use_banks: bool,
) -> anyhow::Result<()> {
    let (bank, program) = TuningData::tuning_program(raw_tuning, use_banks)?;
    let delta = 0.into();
    for (code, value) in [(4, bank), (3, Some(program))] {
        let Some(value) = value else {
            continue;
        };
        track.append(&mut vec![
            // Select RPN (registered parameter number) MSB, then LSB for the code. Parameter 3
            // selects the tuning program. Parameter 4 sets the bank.
            TrackEvent {
                delta,
                kind: TrackEventKind::Midi {
                    channel,
                    message: MidiMessage::Controller {
                        controller: 101.into(),
                        value: 0.into(),
                    },
                },
            },
            TrackEvent {
                delta,
                kind: TrackEventKind::Midi {
                    channel,
                    message: MidiMessage::Controller {
                        controller: 100.into(),
                        value: code.into(),
                    },
                },
            },
            // Set the value using Data Entry MSB (6) per MTS spec
            TrackEvent {
                delta,
                kind: TrackEventKind::Midi {
                    channel,
                    message: MidiMessage::Controller {
                        controller: 6.into(),
                        value,
                    },
                },
            },
        ]);
    }
    Ok(())
}

pub(crate) fn generate(timeline: &Timeline, out: impl AsRef<Path>) -> anyhow::Result<()> {
    let arena = Arena::new();
    let g = MidiGenerator::new(timeline, &arena)?;
    let smf = g.generate()?;
    smf.save(&out)?;
    println!("output written to {}", out.as_ref().display());
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::single_range_in_vec_init)]
    use super::*;

    #[test]
    fn test_split_range() {
        assert_eq!(split_range(0..128, 128), [0..128]);
        assert_eq!(split_range(0..129, 128), [0..65, 65..129]);
        assert_eq!(split_range(0..130, 128), [0..65, 65..130]);
        assert_eq!(split_range(-100..51, 128), [-100..-24, -24..51]);
        assert_eq!(split_range(12..12, 128), []);
    }

    #[test]
    fn test_tuning_data() {
        assert_eq!(TuningData::new(-60..68, 0).midi_offset, 60);
        assert_eq!(TuningData::new(-50..50, 0).midi_offset, 60);
        assert_eq!(TuningData::new(-100..20, 0).midi_offset, 104);
        assert_eq!(TuningData::new(-100..28, 0).midi_offset, 100);
        assert_eq!(TuningData::new(36..164, 0).midi_offset, -36);
    }

    #[test]
    fn test_tuning_program() {
        assert_eq!(
            TuningData::tuning_program(0, false).unwrap(),
            (None, 0.into())
        );
        assert_eq!(
            TuningData::tuning_program(127, false).unwrap(),
            (None, 127.into())
        );
        assert!(TuningData::tuning_program(128, false).is_err());
        assert_eq!(
            TuningData::tuning_program(1, true).unwrap(),
            (Some(1.into()), 1.into())
        );
        assert_eq!(
            TuningData::tuning_program(127, true).unwrap(),
            (Some(1.into()), 127.into())
        );
        assert_eq!(
            TuningData::tuning_program(128, true).unwrap(),
            (Some(2.into()), 0.into())
        );
        assert_eq!(
            TuningData::tuning_program(255, true).unwrap(),
            (Some(2.into()), 127.into())
        );
        assert_eq!(
            TuningData::tuning_program(256, true).unwrap(),
            (Some(3.into()), 0.into())
        );
        assert_eq!(
            TuningData::tuning_program(16255, true).unwrap(),
            (Some(127.into()), 127.into())
        );
        assert!(TuningData::tuning_program(16256, true).is_err());
    }

    #[test]
    fn test_string_exact_bytes() {
        assert_eq!(string_exact_bytes("potato", 10), b"potato    ");
        assert_eq!(
            string_exact_bytes("01𝄞6πα", 10),
            b"01\xf0\x9d\x84\x9e6\xcf\x80 "
        );
    }

    #[test]
    fn test_div_mod_floor() {
        // div_mod_floor is provided by num-integer. This test is just to verify my understanding
        // of it.
        assert_eq!((-13).div_mod_floor(&10), (-2, 7));
        assert_eq!((-7).div_mod_floor(&10), (-1, 3));
        assert_eq!(0.div_mod_floor(&10), (0, 0));
        assert_eq!(2.div_mod_floor(&10), (0, 2));
        assert_eq!(16.div_mod_floor(&10), (1, 6));
    }

    #[test]
    fn test_ramp() {
        assert_eq!(
            ramp(10, 20, 100, 7),
            [
                (14, 11),
                (28, 12),
                (42, 14),
                (57, 15),
                (71, 17),
                (85, 18),
                (100, 20)
            ]
        );
        assert_eq!(
            ramp(20, 10, 100, 7),
            [
                (14, 18),
                (28, 17),
                (42, 15),
                (57, 14),
                (71, 12),
                (85, 11),
                (100, 10)
            ]
        );
        assert_eq!(ramp(10, 12, 100, 7), [(57, 11), (100, 12)]);
        assert_eq!(ramp(10, 12, 100, 0), [(100, 12)]);
    }

    #[test]
    fn test_split_u14() {
        assert!(split_u14(65537).is_err());
        assert!(split_u14(16384).is_err());
        assert_eq!(split_u14(16383).unwrap(), (127.into(), 127.into()));
        assert_eq!(split_u14(128).unwrap(), (1.into(), 0.into()));
        assert_eq!(split_u14(127).unwrap(), (0.into(), 127.into()));
    }

    #[test]
    fn test_bpm_to_micros_per_beat() {
        assert_eq!(
            bpm_to_micros_per_beat(Ratio::from_integer(72)).unwrap(),
            833333
        );
    }

    #[test]
    fn test_ramp_rational() {
        assert_eq!(
            ramp_rational(
                Ratio::new(9, 2),
                Ratio::new(7, 1),
                Ratio::from_integer(12),
                Ratio::new(5, 4),
            ),
            [
                (Ratio::new(49, 4), Ratio::new(5, 1)),
                (Ratio::new(50, 4), Ratio::new(11, 2)),
                (Ratio::new(51, 4), Ratio::new(6, 1)),
                (Ratio::new(52, 4), Ratio::new(13, 2)),
                (Ratio::new(53, 4), Ratio::new(7, 1)),
            ]
        );
    }
}
