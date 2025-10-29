#[cfg(target_family = "windows")]
use crate::controller;
use crate::events;
use crate::events::Event;
use anyhow::anyhow;
#[cfg(target_family = "unix")]
use midir::os::unix::VirtualOutput;
use midir::{MidiOutput, MidiOutputConnection};
use std::collections::HashMap;
use syntoniq_common::pitch::Pitch;
use syntoniq_common::to_anyhow;

struct Player {
    output_connection: MidiOutputConnection,
    pitch_to_channel: HashMap<Pitch, u8>,
    pitch_count: HashMap<Pitch, usize>,
    channels: [bool; 16],
}

impl Player {
    pub fn handle_note(&mut self, pitch: &Pitch, velocity: u8) -> anyhow::Result<()> {
        let (note_number, bend) = pitch.midi().ok_or(anyhow!(
            "unable to convert pitch {pitch} to midi pitch bend"
        ))?;
        let mut ch = self.pitch_to_channel.get(pitch).copied();
        let count = self.pitch_count.get_mut(pitch);
        if velocity == 0 {
            // Remove the note if it was on, freeing the channel if we can
            let Some(count) = count else {
                // This happens if too many notes are on.
                return Ok(());
            };
            *count -= 1;
            if *count == 0 {
                self.pitch_count.remove(pitch);
                let old = self.pitch_to_channel.remove(pitch);
                if let Some(old) = old {
                    log::debug!("midi player: channel {old} is free");
                    self.channels[old as usize] = false;
                }
            }
        } else {
            if ch.is_none() {
                // No channel is associated with this pitch, so allocate one
                for i in 1..16 {
                    if !self.channels[i] {
                        self.channels[i] = true;
                        self.pitch_to_channel.insert(pitch.clone(), i as u8);
                        ch = Some(i as u8);
                        let lsb = (bend & 0x7f) as u8;
                        let msb = (bend >> 7) as u8;
                        self.output_connection.send(&[0xe0 | i as u8, lsb, msb])?;
                        log::debug!("midi player: using channel {i} for pitch {pitch}");
                        break;
                    }
                }
            }
            if ch.is_none() {
                log::warn!("midi player: no available channels; ignoring note operation");
                return Ok(());
            };
            *self.pitch_count.entry(pitch.clone()).or_default() += 1;
        }
        let Some(ch) = ch else {
            // Should not be possible -- ch should always be Some after the above logic.
            log::error!("midi player: ch is None after channel selection; ignoring note");
            return Ok(());
        };
        let op = if velocity == 0 { 0x80 } else { 0x90 };
        self.output_connection
            .send(&[op | ch, note_number, velocity])?;
        Ok(())
    }

    fn init_mpe(&mut self) -> anyhow::Result<()> {
        // Initialize MPE (MIDI Polyphonic Expression) and allocate channels 2-16 (1-15 in our
        // zero-based numbering) for MPE notes.
        let commands: &[&[u8]] = &[
            &[0xB0, 0x65, 0x00], // select MPE (MSB)
            &[0xB0, 0x64, 0x06], // select MPE (LSB)
            &[0xB0, 0x06, 0x0F], // Allocate 15 channels to lower zone
        ];
        for buf in commands {
            self.output_connection.send(buf)?;
        }
        Ok(())
    }
}

pub async fn play_midi(mut events_rx: events::Receiver) -> anyhow::Result<()> {
    // - This works if you do things in the right order.
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
        const PORT_NAME: &str = "Syntoniq Keyboard";
        let midi_out = MidiOutput::new("syntoniq-kbd-midi-out")?;
        #[cfg(target_family = "unix")]
        let output_connection = midi_out.create_virtual(PORT_NAME).map_err(to_anyhow)?;
        #[cfg(target_family = "windows")]
        let output_connection = {
            let out_port = controller::find_port(&midi_out, "syntoniq-loop").inspect_err(|_| {
                eprintln!("Install https://www.tobias-erichsen.de/software/loopmidi.html");
                eprintln!("Create a loop port called \"syntoniq-loop\"");
            })?;
            midi_out.connect(&out_port, PORT_NAME).map_err(to_anyhow)?
        };
        let mut p = Player {
            output_connection,
            pitch_to_channel: Default::default(),
            pitch_count: Default::default(),
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
