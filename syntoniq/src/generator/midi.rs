use anyhow::{anyhow, bail};
use midly::MetaMessage::{EndOfTrack, Tempo};
use midly::num::{u4, u7, u15, u28};
use midly::{
    Arena, Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind,
};
use num_integer::Integer;
use num_rational::Ratio;
use std::cmp;
use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::ops::Range;
use std::path::Path;
use syntoniq_common::parsing::score::{Scale, Tuning};
use syntoniq_common::parsing::{Timeline, TimelineData};
use syntoniq_common::pitch::Pitch;

// Key concepts:
//   - A tuning program consists of at most 128 notes. For the range of notes used in a current
//     scale, there is one tuning program for each 128 notes. A syntoniq tuning is a scale and
//     base pitch. A midi tuning is a subset of a syntoniq tuning.
//   - For simplicity and to allow sustaining notes from one tuning while playing notes from a
//     different tuning, we assign each channel a tuning
//   - A track is a container for events. By convention, all events for a track are for the same
//     port and instrument.
//
// Therefore:
//   - There is one port for every 15 channels (since we avoid channel 9 -- see comments)
//   - There is one (channel, port) pair for each (instrument, tuning)
//   - There is one track for each (instrument, port)
//
// We do the following up front:
//   - Assign a tuning program and, if needed, bank for each tuning.
//   - Dump all tunings to track 0 (midi track 1)
//   - Create a track for each (instrument, port). At the beginning of the track,
//     set the track's instrument and port.
//   - For each (channel, port), get the (instrument, tuning). In the first track for that
//     (port, instrument), set the tuning for that channel to the specific tuning.
//
// Then, when we have a note:
//   - Use the syntoniq tuning and specific note to map to a tuning program.
//   - Use the tuning and instrument to map to a channel and port.
//   - Use the instrument and port to map to a track.
//   - Play the note in the track using the given channel.
//   - The note will have the correct instrument and port because of the track and the correct
//     tuning because of the channel.

#[derive(Debug, PartialOrd, PartialEq, Eq, Hash)]
struct NoteKey<'a> {
    part: &'a str,
    note: u32,
}
struct PlayedNote {
    track: usize,
    channel: u4,
    key: u7,
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
struct PortChannel {
    port: u7,
    channel: u4,
}

struct MidiGenerator<'a> {
    arena: &'a Arena,
    timeline: &'a Timeline,
    scales_by_name: HashMap<&'a str, &'a Scale>,
    last_time: u28,
    ticks_per_beat: u32,
    last_played: HashMap<NoteKey<'a>, PlayedNote>,
    tuning_data: HashMap<&'a Tuning, Vec<TuningData>>,
    channel_data: HashMap<ChannelKey<'a>, PortChannel>,
    track_data: HashMap<TrackKey<'a>, usize>,
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
    instrument: &'a str,
    raw_tuning: i32,
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
struct TrackKey<'a> {
    instrument: &'a str,
    port: u7,
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

impl<'a> MidiGenerator<'a> {
    fn new(timeline: &'a Timeline, arena: &'a Arena) -> Self {
        // Pick a timing that accommodates 2, 3, 5, and 7 as well as anything used by the score.
        let ticks_per_beat = num_integer::lcm(timeline.time_lcm, 210);
        let scales_by_name = timeline
            .scales
            .iter()
            .map(|s| (s.definition.name.as_str(), s.as_ref()))
            .collect();
        Self {
            arena,
            timeline,
            scales_by_name,
            last_time: 0.into(),
            ticks_per_beat,
            last_played: Default::default(),
            tuning_data: Default::default(),
            channel_data: Default::default(),
            track_data: Default::default(),
            tracks: Default::default(),
            smf: None,
        }
    }

    fn get_delta(&mut self, event_time: Ratio<u32>) -> anyhow::Result<u28> {
        // Ticks per beat is known to be a multiple of LCM of all time denominators.
        let time = u28::try_from(*(event_time * self.ticks_per_beat).numer())
            .ok_or_else(|| anyhow!("time overflow"))?;
        if time < self.last_time {
            bail!("time must be monotonically non-decreasing");
        }
        let result = time - self.last_time;
        self.last_time = time;
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
        let mut tunings: HashMap<&Tuning, NoteRange> = HashMap::new();
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
        for (program, (key, range)) in tunings.into_iter().enumerate() {
            let ranges = split_range(range.min_incl..range.max_excl, 128);
            let data = ranges
                .into_iter()
                .map(|range| TuningData::new(range, program as i32 + 1))
                .collect();
            self.tuning_data.insert(key, data);
        }
        Ok(())
    }

    fn get_channel_mappings(&mut self) -> anyhow::Result<()> {
        // Assign a separate channel to each (instrument, tuning) combination. Stay away from
        // channel 9 (from 0) since this is usually used for percussion. Here, we assign a "raw
        // channel", which is converted to a (port, channel) pair.

        let mut channel_users: BTreeSet<ChannelKey> = BTreeSet::new();
        for event in &self.timeline.events {
            let TimelineData::NoteOn(note_event) = &event.data else {
                continue;
            };
            let tuning = self.tuning_for_note(
                &note_event.value.tuning,
                note_event.value.absolute_scale_degree,
            )?;
            let instrument = "TODO"; // TODO: get instrument
            channel_users.insert(ChannelKey {
                instrument,
                raw_tuning: tuning.raw_program,
            });
        }
        let mut port: u7 = 0.into();
        let mut channel: u4 = 0.into();
        let mut too_many = false;
        for channel_key in channel_users {
            if too_many {
                bail!("too many instrument/tuning pairs");
            }
            self.channel_data
                .insert(channel_key, PortChannel { port, channel });
            if channel == 8 {
                // Skip channel 9, percussion
                channel += 2.into();
            } else if channel == 15 {
                channel = 0.into();
                if port == 127 {
                    too_many = true;
                } else {
                    port += 1.into();
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
        let track0 = vec![TrackEvent {
            delta: 0.into(),
            // tempo is microseconds per quarter note
            kind: TrackEventKind::Meta(Tempo(833333.into())),
        }];
        self.tracks.push(track0);
        let mut cur_track = 1usize;
        let mut channels_seen = HashSet::new();
        for (k, port_channel) in &self.channel_data {
            let track_key = TrackKey {
                instrument: k.instrument,
                port: port_channel.port,
            };
            if let Entry::Vacant(v) = self.track_data.entry(track_key) {
                v.insert(cur_track);
                cur_track += 1;
                self.tracks.push(vec![TrackEvent {
                    delta: 0.into(),
                    kind: TrackEventKind::Meta(MetaMessage::MidiPort(port_channel.port)),
                }]);
            }
            if channels_seen.insert(port_channel) {
                // This is the first track for this port/channel, so assign the channel to its
                // fixed tuning program here.
                let track = self.tracks.last_mut().unwrap();
                select_tuning_program(track, port_channel.channel, k.raw_tuning, use_banks)?;
                track.push(TrackEvent {
                    delta: 0.into(),
                    kind: TrackEventKind::Midi {
                        channel: port_channel.channel,
                        // TODO: instrument
                        message: MidiMessage::ProgramChange { program: 23.into() }, // concertina
                    },
                });
            }
        }
        Ok(())
    }

    fn analyze(&mut self) -> anyhow::Result<()> {
        self.get_all_tunings()?;
        self.get_channel_mappings()?;
        self.get_track_assignments()?;
        Ok(())
    }

    fn init_output(&mut self) -> anyhow::Result<()> {
        let Some(metric) = u16::try_from(self.ticks_per_beat)
            .ok()
            .and_then(u15::try_from)
        else {
            bail!(
                "overflow settings ticks per beat; computed value = {}",
                self.ticks_per_beat
            );
        };
        // Timing is ticks per quarter note
        let header = Header::new(Format::Parallel, Timing::Metrical(metric));
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

        let &scale = self
            .scales_by_name
            .get(tuning.scale_name.as_str())
            .ok_or_else(|| anyhow!("unknown scale in dump_tuning"))?;
        let degrees = scale.pitches.len() as i32;
        let cycle_ratio = scale.definition.cycle;
        let base_pitch = &tuning.base_pitch;
        //TODO: if this doesn't cover 128 notes, extend to 0 and 128 as long as we can do so without
        // overflowing or underflowing pitches.
        let first = data.range.start;
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
        let first_midi = data.range.start + data.midi_offset;
        for _ in 0..first_midi {
            dump.append(&mut vec![0; 3]);
        }
        let last_midi = data.range.end + data.midi_offset;
        for _ in first_midi..last_midi {
            let pitch = &pitch0 * &scale.pitches[degree as usize];
            dump.append(
                &mut pitch
                    .midi_tuning()
                    .ok_or_else(|| anyhow!("range error computing midi tuning"))?,
            );
            degree = (degree + 1) % degrees;
            if degree == 0 {
                pitch0 *= &cycle_factor;
            }
        }
        for _ in last_midi..128 {
            dump.append(&mut vec![127; 3]);
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
            // Load 17-EDO into tuning program 1. Tuning program 0 remains 12-TET.
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

    fn generate(mut self) -> anyhow::Result<Smf<'a>> {
        self.analyze()?;
        self.init_output()?;
        self.dump_tunings()?;

        for event in &self.timeline.events {
            let delta = self.get_delta(event.time)?;
            match &event.data {
                TimelineData::NoteOff(e) => {
                    let note_key = NoteKey {
                        part: &e.part,
                        note: e.note_number,
                    };
                    if let Some(last_on) = self.last_played.remove(&note_key) {
                        self.turn_off_last_note(&last_on, delta);
                    } else {
                        eprintln!("TODO: warn about unexpected note")
                    }
                }
                TimelineData::Tuning(_) => {
                    // We don't need to process this. We get all the tuning information up front
                    // based on tuning information in each note. This could be useful in the future
                    // if we ever decide to implement real-time tuning changes. In the initial
                    // design, the tuning is fixed for a channel, which makes it possible to sustain
                    // notes in one channel while playing notes in a different one.
                }
                TimelineData::Dynamic(_) => {} // TODO
                TimelineData::NoteOn(e) => {
                    let note_key = NoteKey {
                        part: &e.part,
                        note: e.note_number,
                    };
                    if let Some(last_on) = self.last_played.remove(&note_key) {
                        // Turn this note off. This would happen if the note were sustained.
                        self.turn_off_last_note(&last_on, delta);
                    }
                    let tuning_data =
                        self.tuning_for_note(&e.value.tuning, e.value.absolute_scale_degree)?;
                    let instrument = "TODO"; // TODO: instrument
                    let port_channel = self
                        .channel_data
                        .get(&ChannelKey {
                            instrument,
                            raw_tuning: tuning_data.raw_program,
                        })
                        .cloned()
                        .ok_or_else(|| anyhow!("unknown channel for note"))?;
                    let track_key = TrackKey {
                        instrument,
                        port: port_channel.port,
                    };
                    let track = self
                        .track_data
                        .get(&track_key)
                        .cloned()
                        .ok_or_else(|| anyhow!("unable to get track for note"))?;
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
                    self.tracks[track].push(TrackEvent {
                        delta,
                        kind: TrackEventKind::Midi {
                            channel: port_channel.channel,
                            message: MidiMessage::NoteOn {
                                key,
                                vel: 72.into(),
                            },
                        },
                    })
                }
            }
        }
        for track in self.tracks.iter_mut() {
            let total_duration = track.iter().fold(u28::from(0), |acc, e| acc + e.delta);
            let delta = self.last_time - total_duration;
            track.push(TrackEvent {
                delta,
                kind: TrackEventKind::Meta(EndOfTrack),
            });
        }

        let mut smf = self.smf.take().unwrap();
        smf.tracks = self.tracks;
        Ok(smf)
    }
}

// Notes on tracks, ports, and channels:
// - A track is just a sequence of events. It typically represents a single part.
// - A port is a device that can have up to 16 channels.
// - To specify the port, use a MidiPort Meta event. Just number ports from 0. While a track
//   can contain events for multiple ports, this is not usually done. Set the port as the first
//   track event.
// - A track often has events for multiple channels.
// - If a total piece needs more than 16 channels, use multiple ports.

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
    let g = MidiGenerator::new(timeline, &arena);
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
}
