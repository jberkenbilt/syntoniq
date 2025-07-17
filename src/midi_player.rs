use crate::events::{Event, PlayNoteEvent};
use crate::scale::Note;
use crate::{events, to_anyhow};
use midir::os::unix::VirtualOutput;
use midir::{MidiOutput, MidiOutputConnection};
use std::collections::{HashMap, HashSet};

struct Player {
    output_connection: MidiOutputConnection,
    bend_to_notes: HashMap<u16, HashSet<String>>,
    bend_to_channel: HashMap<u16, u8>,
    channels: [bool; 16],
}

impl Player {
    pub fn handle_note(&mut self, note: &Note, velocity: u8) -> anyhow::Result<()> {
        let (note_number, bend) = note.nearest_pitch_midi;
        let note_id = &note.unique_id;
        let notes = self.bend_to_notes.get_mut(&bend);
        let mut ch = self.bend_to_channel.get(&bend).copied();
        if velocity == 0 {
            // Remove the note if it was on, freeing the channel if we can
            let Some(notes) = notes else {
                // Should not be possible
                log::warn!("midi player: ignoring note off for note we don't know about");
                return Ok(());
            };
            notes.remove(note_id);
            if notes.is_empty() {
                self.bend_to_notes.remove(&bend);
                let old = self.bend_to_channel.remove(&bend);
                if let Some(old) = old {
                    log::warn!("XXX removing {bend} from channel {old}");
                    self.channels[old as usize] = false;
                }
            }
        } else if ch.is_none() {
            // No channel is associated with this bend, so allocate one
            for i in 0..16 {
                if !self.channels[i] {
                    self.channels[i] = true;
                    self.bend_to_channel.insert(bend, i as u8);
                    self.bend_to_notes
                        .entry(bend)
                        .or_default()
                        .insert(note.unique_id.clone());
                    ch = Some(i as u8);
                    log::warn!("XXX {bend} to channel {i}");
                    break;
                }
            }
        }
        let Some(ch) = ch else {
            log::warn!("midi player: no available channels");
            return Ok(());
        };
        log::warn!("XXX {bend}; using channel {ch}");
        let lsb = (bend & 0x7f) as u8;
        let msb = (bend >> 7) as u8;
        self.output_connection.send(&[0xe0 | ch, lsb, msb])?;
        self.output_connection
            .send(&[0x90 | ch, note_number, velocity])?;
        Ok(())
    }
}

pub async fn play_midi(mut events_rx: events::Receiver) -> anyhow::Result<()> {
    //TODO
    // - This almost works if you do things in the right order:
    //   - Start in midi mode so the output port exists
    //   - Start Surge-XT and ensure only QLaunchpad is input
    //   - Exit surge before exiting qlaunchpad.
    // - Can we turn off all notes at start and end?
    // - Sometimes, notes continue to play
    // - Surge doesn't really seem happy with this
    let (tx, rx) = flume::unbounded();
    let h = tokio::spawn(async move {
        while let Some(event) = events::receive_check_lag(&mut events_rx, Some("midi player")).await
        {
            let Event::PlayNote(event) = event else {
                continue;
            };
            tx.send_async(event).await.unwrap();
        }
    });

    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let midi_out = MidiOutput::new("q-launchpad")?;
        let output_connection = midi_out.create_virtual("QLaunchPad").map_err(to_anyhow)?;
        let mut p = Player {
            output_connection,
            bend_to_notes: Default::default(),
            bend_to_channel: Default::default(),
            channels: [false; 16],
        };
        while let Ok(PlayNoteEvent { note, velocity }) = rx.recv() {
            p.handle_note(note.as_ref(), velocity)?;
        }
        Ok(())
    })
    .await
    .unwrap()?;
    h.await?;
    Ok(())
}
