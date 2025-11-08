use anyhow::{anyhow, bail};
use midly::MetaMessage::{EndOfTrack, Tempo};
use midly::PitchBend;
use midly::num::{u4, u7, u14, u15, u24, u28};
use midly::{
    Arena, Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind,
};
use num_integer::Integer;
use num_rational::Ratio;
use std::borrow::Cow;
use std::cell::RefCell;
use std::cmp;
use std::collections::btree_map::{Entry, VacantEntry};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::{Display, Formatter};
use std::ops::Range;
use std::path::Path;
use std::sync::Arc;
use syntoniq_common::parsing::score::{Scale, Tuning};
use syntoniq_common::parsing::{
    DynamicEvent, MidiInstrumentNumber, NoteEvent, TempoEvent, Timeline, TimelineData,
    TimelineEvent,
};
use syntoniq_common::pitch::Pitch;

// Key concepts:
//   - A "part" is a syntoniq part, corresponding to a part in the score. A "port" is a MIDI port.
//     To reduce confusion, we will use `score_part` and `midi_port` rather than `part` and `port`.
//   - A track is a container for events. By convention, all events for a track are for the same
//     MIDI port and score part.
//   - There are two ways to represent microtuning in MIDI:
//     - Midi Tuning Specification (MTS) -- creates "tuning programs," which assign a pitch to each
//       MIDI note. This is primary intended for live performance settings where an MTS master
//       use MTS-ESP to tune all connected instruments the same way. Some software MIDI renderers,
//       such as Timidity++, can read MTS data.
//     - MIDI Polyphonic Expression (MPE) -- a set of conventions that assign one note per channel,
//       this allowing certain parameters, such as pitch bend and after-touch, to apply at the note
//       level. This is more suited to batch editing in a Digital Audio Workstation and is what most
//       systems that create Microtonal MIDI files create.
//
// *** For MTS ***
//
//   - A tuning program consists of at most 128 notes. For the range of notes used in a current
//     scale, there is one tuning program for each 128 notes. A syntoniq tuning is a scale and
//     base pitch. A midi tuning is a subset of a syntoniq tuning.
//   - For simplicity and to allow sustaining notes from one tuning while playing notes from a
//     different tuning, we assign each channel a tuning.
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
//
// *** For MPE ***
//
//   - Each port can have at most 15 separate note channels. Channel 9 is not special. Channel 0
//     is the control channel. Pitch bend there is global. Channels 1 through 15 (or whatever is
//     specified in the MPE init message) are all note channels.
//   - Each part has exactly one instrument.
//
// Therefore:
//   - There is one MIDI port for every 15 channels (since we don't put notes on channel 0, the
//     MPE control channel)
//   - There is one channel for each distinct note on each instrument. Technically, we could
//     allocate channels based on pitch, but since the syntoniq score syntax already ensures
//     that each "note" line is monophonic, we have a natural way to assign notes to channels.
//   - To avoid needlessly splitting parts up across ports, if a part has 15 or fewer notes numbers,
//     we keep all its channels on the same port. If we have multiple parts, we can "bin-pack"
//     and combine parts on ports if they have 15 or fewer distinct note numbers.
//
// We do the following up front
//   - Count the distinct note numbers for each part
//   - If a part has more than 15 note numbers, create tracks to use up 15, leaving some left as
//     a remainder.
//   - See if we can bin-pack using a naive algorithm (the general problem is NP-complete) to
//     combine some parts (or remainders) into a single port if they have a combined total of 15 or
//     fewer note numbers
//
// Then, when we have a note:
//   - Use the part and note number to find a note's dedicated channel/track
//   - Use the syntoniq tuning and specific note to get the pitch bend for the note, and apply that
//     pitch bend to the note's channel in that track.
//   - Play the note in the track using the given channel.
//   - The note will have the correct instrument and MIDI port because of the track and the correct
//     pitch because of the channel.

#[derive(Copy, Clone)]
pub enum MidiStyle {
    Mts,
    Mpe,
}
impl Display for MidiStyle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MidiStyle::Mts => write!(f, "MTS"),
            MidiStyle::Mpe => write!(f, "MPE"),
        }
    }
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

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
struct MidiNoteData {
    track: usize,
    midi_port: u7,
    channel: u4,
    key: u7,
    bend: Option<u14>,
}

struct MidiGenerator<'s> {
    arena: &'s Arena,
    timeline: &'s Timeline<'s>,
    last_event_time: Ratio<u32>,
    scales_by_name: BTreeMap<Cow<'s, str>, &'s Scale<'s>>,
    track_last_time: RefCell<BTreeMap<usize, u28>>,
    ticks_per_beat: u15,
    micros_per_beat: u24,
    part_channels: BTreeMap<&'s str, BTreeSet<TrackPortChannel>>,
    tracks: Vec<Vec<TrackEvent<'s>>>,
    pitch_data: PitchData<'s>,
    smf: Option<Smf<'s>>,
}
enum PitchData<'s> {
    Mts(MtsData<'s>),
    Mpe(MpeData<'s>),
}

fn set_channel_instrument(
    midi_instruments: &BTreeMap<Cow<str>, MidiInstrumentNumber>,
    track: &mut Vec<TrackEvent>,
    score_part: &str,
    channel: u4,
) -> anyhow::Result<()> {
    let Some(instrument) = midi_instruments
        .get(score_part)
        .or_else(|| midi_instruments.get(""))
    else {
        return Ok(());
    };
    if instrument.bank > 0 {
        let (bank_msb, bank_lsb) = split_u14(instrument.bank)?;
        track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Midi {
                channel,
                message: MidiMessage::Controller {
                    controller: 0.into(), // Bank Select MSB
                    value: bank_msb,
                },
            },
        });
        track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Midi {
                channel,
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
            channel,
            message: MidiMessage::ProgramChange { program },
        },
    });
    Ok(())
}

struct MtsData<'s> {
    tuning_data: BTreeMap<&'s Tuning<'s>, Vec<TuningData>>,
    channel_data: BTreeMap<MtsChannelKey<'s>, PortChannel>,
    track_data: BTreeMap<MtsTrackKey<'s>, usize>,
}
impl<'s> MtsData<'s> {
    fn tuning_for_note(&'s self, key: &'s Tuning<'s>, note: i32) -> anyhow::Result<&'s TuningData> {
        let data = self
            .tuning_data
            .get(key)
            .ok_or_else(|| anyhow!("no tuning data for known tuning"))?;
        data.iter()
            .find(|&i| i.range.contains(&note))
            .ok_or_else(|| anyhow!("internal error unable to get tuning data for note"))
    }

    fn track_port_channel_key(
        &self,
        score_part: &str,
        note_event: &NoteEvent,
    ) -> anyhow::Result<MidiNoteData> {
        let tuning_data = self.tuning_for_note(
            &note_event.value.tuning,
            note_event.value.absolute_scale_degree,
        )?;
        let port_channel = self
            .channel_data
            .get(&MtsChannelKey {
                score_part,
                raw_tuning: tuning_data.raw_program,
            })
            .cloned()
            .ok_or_else(|| anyhow!("unknown channel for note"))?;
        let key = u8::try_from(note_event.value.absolute_scale_degree + tuning_data.midi_offset)
            .ok()
            .and_then(u7::try_from)
            .ok_or_else(|| anyhow!("internal error: note out of range"))?;
        let track_key = MtsTrackKey {
            score_part,
            midi_port: port_channel.midi_port,
        };
        let track = self
            .track_data
            .get(&track_key)
            .cloned()
            .ok_or_else(|| anyhow!("unable to get track for note"))?;
        Ok(MidiNoteData {
            track,
            midi_port: port_channel.midi_port,
            channel: port_channel.channel,
            key,
            bend: None,
        })
    }

    fn get_channel_mappings(
        &mut self,
        events: &'s BTreeSet<Arc<TimelineEvent<'s>>>,
    ) -> anyhow::Result<()> {
        // Assign a separate channel to each (score_part, tuning) combination. Stay away from
        // channel 9 (from 0) since this is usually used for percussion. Here, we assign a "raw
        // channel", which is converted to a (midi_port, channel) pair.
        self.get_all_tunings(events)?;
        let mut channel_users: BTreeSet<MtsChannelKey> = BTreeSet::new();
        for event in events {
            let TimelineData::Note(note_event) = &event.data else {
                continue;
            };
            let tuning = self.tuning_for_note(
                &note_event.value.tuning,
                note_event.value.absolute_scale_degree,
            )?;
            let score_part = note_event.part_note.part;
            channel_users.insert(MtsChannelKey {
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

    fn get_all_tunings(
        &mut self,
        events: &'s BTreeSet<Arc<TimelineEvent<'s>>>,
    ) -> anyhow::Result<()> {
        // A given tuning may have up to 128 notes (midi note numbers 0 through 127). For each
        // scale, figure the range of notes used, and divide into tunings.
        let mut tunings: BTreeMap<&Tuning<'s>, NoteRange> = BTreeMap::new();
        for event in events {
            let TimelineData::Note(note_event) = &event.data else {
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
        let mut tunings: Vec<(&Tuning<'s>, NoteRange)> = tunings
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

    fn use_banks(&self) -> bool {
        self.tuning_data.len() > 126
    }

    fn get_track_assignments(
        &mut self,
        midi_instruments: &BTreeMap<Cow<str>, MidiInstrumentNumber>,
        tracks: &mut Vec<Vec<TrackEvent>>,
    ) -> anyhow::Result<()> {
        let mut cur_track = 1usize;
        let mut channels_seen = BTreeSet::new();
        let use_banks = self.use_banks();
        for (k, port_channel) in &self.channel_data {
            let track_key = MtsTrackKey {
                score_part: k.score_part,
                midi_port: port_channel.midi_port,
            };
            if let Entry::Vacant(v) = self.track_data.entry(track_key) {
                add_track(v, tracks, &mut cur_track, port_channel.midi_port);
            }
            if channels_seen.insert(port_channel) {
                // This is the first time this channel has been seen. Initialize its instrument and
                // tuning program.
                let track = tracks.last_mut().unwrap();
                select_tuning_program(track, port_channel.channel, k.raw_tuning, use_banks)?;
                set_channel_instrument(
                    midi_instruments,
                    track,
                    k.score_part,
                    port_channel.channel,
                )?;
            }
        }
        Ok(())
    }

    fn dump_tuning(
        arena: &'s Arena,
        scales_by_name: &BTreeMap<Cow<'s, str>, &'s Scale<'s>>,
        track: &mut Vec<TrackEvent<'s>>,
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
        let &scale = scales_by_name
            .get(&tuning.scale_name)
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
        //TODO: overflow detection is a little shaky. There are likely conditions that will
        // cause pitch overflow/underflow conditions or very sparse scales are crazy base pitches.
        let mut hit_max = false;
        for i in 0..128 {
            let pitch = &pitch0 * &scale.pitches[degree as usize];
            let mut v = pitch.midi_tuning().unwrap_or_else(|| {
                if i < data.midi_offset {
                    vec![0; 3]
                } else {
                    // No more cycle advances will happen -- from here, we'll keep repeating
                    // pitch classes in the top octave.
                    hit_max = true;
                    vec![127; 3]
                }
            });
            dump.append(&mut v);
            degree = (degree + 1) % degrees;
            if degree == 0 && !hit_max {
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

    fn dump_tunings(
        &mut self,
        arena: &'s Arena,
        scales_by_name: &BTreeMap<Cow<'s, str>, &'s Scale<'s>>,
        track: &mut Vec<TrackEvent<'s>>,
    ) -> anyhow::Result<()> {
        let use_banks = self.use_banks();
        let mut tunings: Vec<_> = self.tuning_data.iter().collect();
        tunings.sort_by_key(|x| x.0);
        let mut events = Vec::new();
        for (&tuning, tuning_data_vec) in tunings {
            for tuning_data in tuning_data_vec {
                Self::dump_tuning(
                    arena,
                    scales_by_name,
                    &mut events,
                    tuning,
                    tuning_data,
                    use_banks,
                )?;
            }
        }
        track.append(&mut events);
        Ok(())
    }

    fn get_part_channels(
        &mut self,
        part_channels: &mut BTreeMap<&'s str, BTreeSet<TrackPortChannel>>,
    ) -> anyhow::Result<()> {
        // For each distinct part, make a list of all the tracks it uses. This is needed for
        // dynamics.
        for (channel_key, port_channel) in &self.channel_data {
            let track_key = MtsTrackKey {
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
            part_channels
                .entry(channel_key.score_part)
                .or_default()
                .insert(tpc);
        }
        Ok(())
    }
}

struct MpeData<'s> {
    channel_data: BTreeMap<MpeChannelKey<'s>, PortChannel>,
    track_data: BTreeMap<MpeTrackKey<'s>, usize>,
}
impl<'s> MpeData<'s> {
    fn get_channel_mappings(
        &mut self,
        events: &'s BTreeSet<Arc<TimelineEvent<'s>>>,
    ) -> anyhow::Result<()> {
        // Assign a separate channel for each note for MPE by first creating bins of parts and
        // notes and then assigning a port to each bin.
        let mut channels_for_part: BTreeMap<&str, BTreeSet<u32>> = BTreeMap::new();
        for event in events {
            let TimelineData::Note(note_event) = &event.data else {
                continue;
            };
            channels_for_part
                .entry(note_event.part_note.part)
                .or_default()
                .insert(note_event.part_note.note_number);
        }
        let mut all_items: Vec<(&str, VecDeque<u32>)> = Default::default();
        for (score_part, channels_set) in channels_for_part {
            all_items.push((score_part, channels_set.into_iter().collect()));
        }
        let bins = bin_pack(15, all_items);
        for (i, bin) in bins.into_iter().enumerate() {
            let midi_port = u7::from(i as u8);
            for (ch, (score_part, note_number)) in bin.into_iter().enumerate() {
                let key = MpeChannelKey {
                    score_part,
                    note_number,
                };
                let port_channel = PortChannel {
                    midi_port,
                    channel: u4::from((1 + ch) as u8),
                };
                self.channel_data.insert(key, port_channel);
            }
        }
        Ok(())
    }

    fn get_track_assignments(
        &mut self,
        midi_instruments: &BTreeMap<Cow<str>, MidiInstrumentNumber>,
        tracks: &mut Vec<Vec<TrackEvent>>,
    ) -> anyhow::Result<()> {
        let mut cur_track = 1usize;
        let mut channels_seen = BTreeSet::new();
        let mut ports_seen = BTreeSet::new();
        for (k, port_channel) in &self.channel_data {
            let track_key = MpeTrackKey {
                score_part: k.score_part,
                channel: port_channel.channel,
            };
            if let Entry::Vacant(v) = self.track_data.entry(track_key) {
                add_track(v, tracks, &mut cur_track, port_channel.midi_port);
            }
            if channels_seen.insert(port_channel) {
                let track = tracks.last_mut().unwrap();
                set_channel_instrument(
                    midi_instruments,
                    track,
                    k.score_part,
                    port_channel.channel,
                )?;
            }
            if ports_seen.insert(port_channel.midi_port) {
                // This is the first time we've seen this port, so use this track to initialize
                // MPE for the port.
                let track = tracks.last_mut().unwrap();
                init_mpe(track);
            }
        }
        Ok(())
    }

    fn get_part_channels(
        &mut self,
        part_channels: &mut BTreeMap<&'s str, BTreeSet<TrackPortChannel>>,
    ) -> anyhow::Result<()> {
        // For each distinct part, make a list of all the tracks it uses. This is needed for
        // dynamics.
        for (channel_key, port_channel) in &self.channel_data {
            let track_key = MpeTrackKey {
                score_part: channel_key.score_part,
                channel: port_channel.channel,
            };
            let &track = self.track_data.get(&track_key).ok_or_else(|| {
                anyhow!("get_part_channels: unable to get track for score_part/midi_port")
            })?;
            let tpc = TrackPortChannel {
                track,
                midi_port: port_channel.midi_port,
                channel: port_channel.channel,
            };
            part_channels
                .entry(channel_key.score_part)
                .or_default()
                .insert(tpc);
        }
        Ok(())
    }

    fn track_port_channel_key(
        &self,
        score_part: &str,
        note_event: &NoteEvent,
    ) -> anyhow::Result<MidiNoteData> {
        let port_channel = self
            .channel_data
            .get(&MpeChannelKey {
                score_part,
                note_number: note_event.part_note.note_number,
            })
            .cloned()
            .ok_or_else(|| anyhow!("unknown channel for note"))?;
        let (midi_note, bend) = note_event
            .value
            .absolute_pitch
            .midi()
            .ok_or_else(|| anyhow!("error getting MIDI pitch information for pitch"))?;
        let track_key = MpeTrackKey {
            score_part,
            channel: port_channel.channel,
        };
        let track = self
            .track_data
            .get(&track_key)
            .cloned()
            .ok_or_else(|| anyhow!("unable to get track for note"))?;
        Ok(MidiNoteData {
            track,
            midi_port: port_channel.midi_port,
            channel: port_channel.channel,
            key: midi_note.into(),
            bend: Some(bend.into()),
        })
    }
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
struct MtsChannelKey<'a> {
    score_part: &'a str,
    raw_tuning: i32,
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
struct MpeChannelKey<'a> {
    score_part: &'a str,
    note_number: u32,
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
struct MtsTrackKey<'a> {
    score_part: &'a str,
    midi_port: u7,
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
struct MpeTrackKey<'a> {
    score_part: &'a str,
    channel: u4,
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

/// Given a group labelled groups `(A, [B])`, pack these into bins of `[A, B]` of no more than a
/// given size. This is used to pack part/note pairs into groups of channels. See the test case for
/// details.
fn bin_pack<A: Copy, B>(max_size: usize, items: Vec<(A, VecDeque<B>)>) -> Vec<Vec<(A, B)>> {
    assert!((0..127).contains(&max_size));
    let mut leftovers: Vec<(A, Vec<B>)> = Default::default();
    let mut bins: Vec<Vec<(A, B)>> = Default::default();
    // In each group, take initial subgroups of max_size and fill bins with them.
    for (a, mut b_group) in items {
        while b_group.len() >= max_size {
            let group = b_group.drain(..max_size);
            bins.push(Default::default());
            let bin = bins.last_mut().unwrap();
            for b in group {
                bin.push((a, b));
            }
        }
        // Throw whatever's left in a pile for later aggregation.
        leftovers.push((a, b_group.into_iter().collect()));
    }
    // Pack the leftovers into groups, combining as we can. Doing this optimally is NP-complete, so
    // we use a simple heuristic of taking remaining groups in decreasing order of side and placing
    // them into whichever bin the fit most tightly. Don't worry about the runtime efficiency -- we
    // use this on groups of parts and notes, which will be small.
    let mut remainders: Vec<Vec<(A, B)>> = Default::default();
    leftovers.sort_by_key(|x| -(x.1.len() as i8));
    for (a, b_group) in leftovers {
        let mut best_idx = 0;
        let mut best_remainder = max_size;
        let mut found = false;
        for (i, bin) in remainders.iter().enumerate() {
            let new_size = bin.len() + b_group.len();
            if new_size > max_size {
                continue;
            }
            let remainder = max_size - new_size;
            if remainder < best_remainder {
                best_remainder = remainder;
                best_idx = i;
                found = true;
            }
        }
        if !found {
            remainders.push(Vec::new());
            best_idx = remainders.len() - 1;
        }
        for b in b_group {
            remainders[best_idx].push((a, b));
        }
    }
    bins.append(&mut remainders);
    bins
}

fn bpm_to_micros_per_beat(bpm: Ratio<u32>) -> anyhow::Result<u24> {
    let &micros_per_beat = (Ratio::from_integer(60_000_000) / bpm).floor().numer();
    u24::try_from(micros_per_beat).ok_or_else(|| anyhow!("overflow calculating tempo"))
}

impl<'s> MidiGenerator<'s> {
    fn new(timeline: &'s Timeline, arena: &'s Arena, style: MidiStyle) -> anyhow::Result<Self> {
        // Pick a timing that accommodates 2, 3, 5, and 7 as well as anything used by the score.
        let ticks_per_beat = u16::try_from(num_integer::lcm(timeline.time_lcm, 210))
            .ok()
            .and_then(u15::try_from)
            .ok_or_else(|| anyhow!("overflow calculating ticks per beat"))?;
        let micros_per_beat: u24 = 833333.into(); // 72 BPM -- changed by tempo events
        let scales_by_name = timeline
            .scales
            .iter()
            .map(|s| (s.definition.name.clone(), s.as_ref()))
            .collect();
        let pitch_data = match style {
            MidiStyle::Mts => PitchData::Mts(MtsData {
                tuning_data: Default::default(),
                channel_data: Default::default(),
                track_data: Default::default(),
            }),
            MidiStyle::Mpe => PitchData::Mpe(MpeData {
                channel_data: Default::default(),
                track_data: Default::default(),
            }),
        };
        Ok(Self {
            arena,
            timeline,
            last_event_time: Ratio::from_integer(0),
            scales_by_name,
            track_last_time: Default::default(),
            ticks_per_beat,
            micros_per_beat,
            pitch_data,
            part_channels: Default::default(),
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

    fn init_tracks(&self) -> Vec<Vec<TrackEvent<'s>>> {
        // A track should contain only notes for a single instrument/port. The first track
        // is for global information.
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
        vec![track0]
    }

    fn analyze(&mut self) -> anyhow::Result<()> {
        let mut tracks = self.init_tracks();
        match &mut self.pitch_data {
            PitchData::Mts(mts) => {
                mts.get_channel_mappings(&self.timeline.events)?;
                mts.get_track_assignments(&self.timeline.midi_instruments, &mut tracks)?;
                mts.get_part_channels(&mut self.part_channels)?;
            }
            PitchData::Mpe(mpe) => {
                mpe.get_channel_mappings(&self.timeline.events)?;
                mpe.get_track_assignments(&self.timeline.midi_instruments, &mut tracks)?;
                mpe.get_part_channels(&mut self.part_channels)?;
            }
        };
        self.tracks = tracks;
        Ok(())
    }

    fn init_output(&mut self) -> anyhow::Result<()> {
        let header = Header::new(Format::Parallel, Timing::Metrical(self.ticks_per_beat));
        self.smf = Some(Smf::new(header));
        Ok(())
    }

    fn volume_event(tpc: TrackPortChannel, delta: u28, value: u7) -> TrackEvent<'s> {
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

    fn handle_tempo_event(
        &mut self,
        events: &mut BTreeSet<Arc<TimelineEvent<'s>>>,
        event: &TimelineEvent<'s>,
        tempo_event: &TempoEvent,
    ) -> anyhow::Result<()> {
        self.micros_per_beat = bpm_to_micros_per_beat(tempo_event.bpm)?;
        // All tempo events go in track 0.
        let delta = self.get_delta(0, event.time)?;
        self.tracks[0].push(TrackEvent {
            delta,
            kind: TrackEventKind::Meta(Tempo(self.micros_per_beat)),
        });
        if let Some(t) = &tempo_event.end_bpm {
            let end_bpm = t.item;
            // The event comes with an absolute time. We need a duration.
            let duration = t.time - event.time;
            for (time, bpm) in ramp_rational(tempo_event.bpm, end_bpm, event.time, duration) {
                events.insert(Arc::new(TimelineEvent {
                    time,
                    repeat_depth: event.repeat_depth,
                    span: event.span,
                    data: TimelineData::Tempo(TempoEvent { bpm, end_bpm: None }),
                }));
            }
        }
        Ok(())
    }

    fn handle_dynamic_event(
        &mut self,
        events: &mut BTreeSet<Arc<TimelineEvent<'s>>>,
        event: &TimelineEvent<'s>,
        dynamic_event: &DynamicEvent<'s>,
    ) -> anyhow::Result<()> {
        let part_channels = self
            .part_channels
            .get(dynamic_event.part)
            .ok_or_else(|| anyhow!("unable to get part channels"))?;
        for &tpc in part_channels {
            let delta = self.get_delta(tpc.track, event.time)?;
            let value = u7::try_from(dynamic_event.start_level)
                .ok_or_else(|| anyhow!("volume out of range"))?;
            self.tracks[tpc.track].push(Self::volume_event(tpc, delta, value));
            if let Some(end_level) = &dynamic_event.end_level {
                let total_time = end_level.time - event.time;
                let total_ticks = *(total_time * u16::from(self.ticks_per_beat) as u32)
                    .floor()
                    .numer();
                let steps = 10;
                for (ticks, level) in ramp(
                    dynamic_event.start_level,
                    end_level.item,
                    total_ticks,
                    steps,
                ) {
                    let time = event.time + (Ratio::new(ticks, total_ticks) * total_time);
                    events.insert(Arc::new(TimelineEvent {
                        time,
                        repeat_depth: event.repeat_depth,
                        span: event.span,
                        data: TimelineData::Dynamic(DynamicEvent {
                            text: dynamic_event.text,
                            part: dynamic_event.part,
                            start_level: level,
                            end_level: None,
                        }),
                    }));
                }
            }
        }
        Ok(())
    }

    fn handle_note_event(
        &mut self,
        events: &mut BTreeSet<Arc<TimelineEvent<'s>>>,
        event: &TimelineEvent<'s>,
        note_event: &NoteEvent<'s>,
    ) -> anyhow::Result<()> {
        let score_part = note_event.part_note.part;
        let midi_note = match &self.pitch_data {
            PitchData::Mts(mts) => mts.track_port_channel_key(score_part, note_event)?,
            PitchData::Mpe(mpe) => mpe.track_port_channel_key(score_part, note_event)?,
        };
        let velocity = u7::try_from(note_event.value.velocity)
            .ok_or_else(|| anyhow!("overflow getting velocity"))?;
        let mut delta = self.get_delta(midi_note.track, event.time)?;
        if velocity == 0 {
            self.tracks[midi_note.track].push(TrackEvent {
                delta,
                kind: TrackEventKind::Midi {
                    channel: midi_note.channel,
                    message: MidiMessage::NoteOff {
                        key: midi_note.key,
                        vel: velocity,
                    },
                },
            });
        } else {
            if let Some(bend) = midi_note.bend {
                self.tracks[midi_note.track].push(TrackEvent {
                    delta,
                    kind: TrackEventKind::Midi {
                        channel: midi_note.channel,
                        message: MidiMessage::PitchBend {
                            bend: PitchBend(bend),
                        },
                    },
                });
                delta = 0.into();
            }
            self.tracks[midi_note.track].push(TrackEvent {
                delta,
                kind: TrackEventKind::Midi {
                    channel: midi_note.channel,
                    message: MidiMessage::NoteOn {
                        key: midi_note.key,
                        vel: velocity,
                    },
                },
            });
            // Generate an event to turn the note off. Use velocity 0 as a signal.
            let mut off = note_event.clone();
            off.value.velocity = 0;
            let off_event = Arc::new(TimelineEvent {
                time: note_event.value.adjusted_end_time,
                repeat_depth: event.repeat_depth,
                span: event.span,
                data: TimelineData::Note(off),
            });
            events.insert(off_event);
        }
        Ok(())
    }

    fn handle_event(
        &mut self,
        events: &mut BTreeSet<Arc<TimelineEvent<'s>>>,
        event: &TimelineEvent<'s>,
    ) -> anyhow::Result<()> {
        // We have to track last event time as we go since events may be inserted into the
        // even stream during processing.
        self.last_event_time = event.time;
        match &event.data {
            TimelineData::Tempo(e) => self.handle_tempo_event(events, event, e)?,
            TimelineData::Dynamic(e) => self.handle_dynamic_event(events, event, e)?,
            TimelineData::Note(e) => self.handle_note_event(events, event, e)?,
            TimelineData::Mark(_) | TimelineData::RepeatStart(_) | TimelineData::RepeatEnd(_) => {}
        }
        Ok(())
    }

    fn generate(mut self) -> anyhow::Result<Smf<'s>> {
        self.analyze()?;
        self.init_output()?;
        match &mut self.pitch_data {
            PitchData::Mts(mts) => {
                mts.dump_tunings(self.arena, &self.scales_by_name, &mut self.tracks[0])?
            }
            PitchData::Mpe(_) => {}
        }

        let mut events: BTreeSet<_> = self.timeline.events.iter().cloned().collect();
        while let Some(event) = events.pop_first() {
            if let Err(e) = self.handle_event(&mut events, &event) {
                bail!("while handle event at location {}: {e}", event.span);
            }
        }
        let deltas: Vec<_> = (0..self.tracks.len())
            .map(|track| self.get_delta(track, self.last_event_time).unwrap())
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

fn add_track<T: Ord>(
    v: VacantEntry<T, usize>,
    tracks: &mut Vec<Vec<TrackEvent>>,
    cur_track: &mut usize,
    midi_port: u7,
) {
    v.insert(*cur_track);
    *cur_track += 1;
    tracks.push(vec![TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::MidiPort(midi_port)),
    }]);
}

fn end_rpn(track: &mut Vec<TrackEvent>, channel: u4) {
    set_midi_parameter(track, 0.into(), channel, 16383.into(), None, None);
}

fn set_midi_parameter(
    track: &mut Vec<TrackEvent>,
    delta: u28,
    channel: u4,
    param: u14,
    value_msb: Option<u7>,
    value_lsb: Option<u7>,
) {
    let (msb, lsb) = split_u14(param).unwrap();
    track.append(&mut vec![
        // Select RPN (registered parameter number): code MSB, then code LSB, then 6 for data entry
        // with value.
        TrackEvent {
            delta,
            kind: TrackEventKind::Midi {
                channel,
                message: MidiMessage::Controller {
                    controller: 101.into(), // RPN MSB
                    value: msb,
                },
            },
        },
        TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Midi {
                channel,
                message: MidiMessage::Controller {
                    controller: 100.into(), // RPN LSB
                    value: lsb,
                },
            },
        },
    ]);
    // Set the value using Data Entry MSB (6) and LSB (38)
    if let Some(value_msb) = value_msb {
        track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Midi {
                channel,
                message: MidiMessage::Controller {
                    controller: 6.into(), // data entry, msb
                    value: value_msb,
                },
            },
        });
    }
    if let Some(value_lsb) = value_lsb {
        track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Midi {
                channel,
                message: MidiMessage::Controller {
                    controller: 38.into(), // data entry, lsb
                    value: value_lsb,
                },
            },
        });
    }
}

fn select_tuning_program(
    track: &mut Vec<TrackEvent>,
    channel: u4,
    raw_tuning: i32,
    use_banks: bool,
) -> anyhow::Result<()> {
    let (bank, program) = TuningData::tuning_program(raw_tuning, use_banks)?;
    // Select RPN (registered parameter number) MSB, then LSB for the code. Parameter 3
    // selects the tuning program. Parameter 4 sets the bank.
    for (param, value) in [(4, bank), (3, Some(program))] {
        let Some(value) = value else {
            continue;
        };
        set_midi_parameter(track, 0.into(), channel, param.into(), Some(value), None);
    }
    end_rpn(track, channel);
    Ok(())
}

fn init_mpe(track: &mut Vec<TrackEvent>) {
    // Initialize MPE for a single "low" zone with 15 channels.
    set_midi_parameter(track, 0.into(), 0.into(), 6.into(), Some(15.into()), None);
    // Set pitch bend for channel 0 to 2 semitones.
    set_midi_parameter(track, 0.into(), 0.into(), 0.into(), Some(2.into()), None);
    end_rpn(track, 0.into());
    for ch in 1..=15 {
        // Explicitly set pitch bend sensitivity to 48 semitones for all the other channels. This
        // can help with instruments that support pitch bend but are not necessarily MPE-aware.
        set_midi_parameter(
            track,
            0.into(),
            ch.into(),
            0.into(),
            Some(48.into()),
            Some(0.into()),
        );
        end_rpn(track, ch.into());
    }
}

pub(crate) fn generate(
    timeline: &Timeline,
    out: impl AsRef<Path>,
    style: MidiStyle,
) -> anyhow::Result<()> {
    let arena = Arena::new();
    let g = MidiGenerator::new(timeline, &arena, style)?;
    let smf = g.generate()?;
    smf.save(&out)?;
    println!(
        "MIDI output ({style}) written to {}",
        out.as_ref().display()
    );
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

    #[test]
    fn test_bin_pack() {
        let mut orig: Vec<(&str, VecDeque<i32>)> = Default::default();
        for (a, b_max) in [("a", 2), ("b", 9), ("c", 3), ("d", 2), ("e", 2), ("f", 4)] {
            orig.push((a, (0..b_max).collect()));
        }
        let bins = bin_pack(6, orig);
        assert_eq!(
            bins,
            vec![
                vec![("b", 0), ("b", 1), ("b", 2), ("b", 3), ("b", 4), ("b", 5),],
                vec![("f", 0), ("f", 1), ("f", 2), ("f", 3), ("a", 0), ("a", 1),],
                vec![("b", 6), ("b", 7), ("b", 8), ("c", 0), ("c", 1), ("c", 2),],
                vec![("d", 0), ("d", 1), ("e", 0), ("e", 1),],
            ]
        );

        let mut orig: Vec<(&str, VecDeque<i32>)> = Default::default();
        for (a, b_max) in [("a", 4), ("b", 3), ("c", 2)] {
            orig.push((a, (0..b_max).collect()));
        }
        let bins = bin_pack(6, orig);
        assert_eq!(
            bins,
            vec![
                vec![("a", 0), ("a", 1), ("a", 2), ("a", 3), ("c", 0), ("c", 1),],
                vec![("b", 0), ("b", 1), ("b", 2),],
            ]
        );
    }
}
