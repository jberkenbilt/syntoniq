use crate::events;
use crate::events::Event;
use anyhow::anyhow;
use midir::os::unix::VirtualOutput;
use midir::{MidiOutput, MidiOutputConnection};
use std::collections::HashMap;
use syntoniq_common::pitch::Pitch;
use syntoniq_common::to_anyhow;

struct Player {
    output_connection: MidiOutputConnection,
    bend_to_notes: HashMap<u16, HashMap<Pitch, u8>>,
    bend_to_channel: HashMap<u16, u8>,
    channels: [bool; 16],
}

impl Player {
    pub fn handle_note(&mut self, pitch: &Pitch, velocity: u8) -> anyhow::Result<()> {
        let (note_number, bend) = pitch.midi().ok_or(anyhow!(
            "unable to convert pitch {pitch} to midi pitch bend"
        ))?;
        let notes = self.bend_to_notes.get_mut(&bend);
        let mut ch = self.bend_to_channel.get(&bend).copied();
        if velocity == 0 {
            // Remove the note if it was on, freeing the channel if we can
            let Some(notes) = notes else {
                // Should not be possible
                log::warn!("midi player: no notes for {bend}; ignoring off for {pitch}");
                return Ok(());
            };
            let Some(count) = notes.get_mut(pitch) else {
                log::warn!("midi player: no count for {pitch}; ignoring off");
                return Ok(());
            };
            *count -= 1;
            if *count == 0 {
                notes.remove(pitch);
            }
            if notes.is_empty() {
                self.bend_to_notes.remove(&bend);
                let old = self.bend_to_channel.remove(&bend);
                if let Some(old) = old {
                    log::debug!("midi player: channel {old} is free");
                    self.channels[old as usize] = false;
                }
            }
        } else {
            if ch.is_none() {
                // No channel is associated with this bend, so allocate one
                for i in 0..16 {
                    if !self.channels[i] {
                        self.channels[i] = true;
                        self.bend_to_channel.insert(bend, i as u8);
                        ch = Some(i as u8);
                        let lsb = (bend & 0x7f) as u8;
                        let msb = (bend >> 7) as u8;
                        self.output_connection.send(&[0xe0 | i as u8, lsb, msb])?;
                        log::debug!("midi player: using channel {i} for bend {bend}");
                        break;
                    }
                }
            }
            if ch.is_none() {
                log::warn!("midi player: no available channels; ignoring note operation");
                return Ok(());
            };
            *self
                .bend_to_notes
                .entry(bend)
                .or_default()
                .entry(pitch.clone())
                .or_default() += 1;
        }
        let Some(ch) = ch else {
            // Should not be possible -- ch should always be Some after the above logic.
            log::error!("midi player: ch is None after channel selection; ignoring note");
            return Ok(());
        };
        self.output_connection
            .send(&[0x90 | ch, note_number, velocity])?;
        Ok(())
    }

    fn init_mpe(&mut self) -> anyhow::Result<()> {
        // Initialize MPE (MIDI Polyphonic Expression) and allocate channels 2-16 (1-15 in our
        // zero-based numbering) for MPE notes.
        let commands: &[&[u8]] = &[
            &[0xB0, 0x65, 0x06], // select MPE (MSB)
            &[0xB0, 0x64, 0x00], // select MPE (LSB)
            &[0xB0, 0x06, 0x00], // Data Entry (MSB)
            &[0xB0, 0x26, 0x0E], // Data Entry (LSB)
        ];
        for buf in commands {
            self.output_connection.send(buf)?;
        }
        Ok(())
    }
}

pub async fn play_midi(mut events_rx: events::Receiver) -> anyhow::Result<()> {
    // - This almost works if you do things in the right order, but there seem to be issues with
    //   Surge-XT
    //   - Start in midi mode so the output port exists
    //   - Start Surge-XT and ensure only Syntoniq input
    //   - Exit surge before exiting syntoniq.
    let (tx, rx) = flume::unbounded();
    let h = tokio::spawn(async move {
        while let Some(event) = events::receive_check_lag(&mut events_rx, Some("midi player")).await
        {
            match event {
                Event::SelectLayout(_) => {}
                Event::PlayNote(_) => {}
                _ => continue,
            }
            tx.send_async(event).await.unwrap();
        }
    });

    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let midi_out = MidiOutput::new("q-launchpad")?;
        let output_connection = midi_out.create_virtual("Syntoniq").map_err(to_anyhow)?;
        let mut p = Player {
            output_connection,
            bend_to_notes: Default::default(),
            bend_to_channel: Default::default(),
            channels: [false; 16],
        };
        p.channels[0] = true; // reserve channel 1 -- MPE doesn't expect note on it
        while let Ok(event) = rx.recv() {
            match event {
                Event::PlayNote(e) => p.handle_note(&e.pitch, e.velocity)?,
                Event::SelectLayout(_) => p.init_mpe()?,
                _ => {}
            }
        }
        Ok(())
    })
    .await
    .unwrap()?;
    h.await?;
    Ok(())
}
