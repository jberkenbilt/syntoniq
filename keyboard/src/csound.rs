use crate::csound::wrapper::CsoundApi;
use crate::events;
use crate::events::Event;
use std::collections::HashMap;
use syntoniq_common::pitch::Pitch;

mod wrapper;

struct CSound {
    api: CsoundApi,
    notes: HashMap<Pitch, u32>,
    note_to_number: HashMap<Pitch, u32>,
    number_to_note: HashMap<u32, Pitch>,
}

const CSOUND_FILE: &str = include_str!("sound.csd");

impl CSound {
    pub async fn new(events_tx: events::WeakSender) -> anyhow::Result<Self> {
        let api = CsoundApi::new(CSOUND_FILE, events_tx).await?;
        Ok(Self {
            api,
            notes: Default::default(),
            note_to_number: Default::default(),
            number_to_note: Default::default(),
        })
    }

    pub async fn handle_note(&mut self, pitch: &Pitch, velocity: u8) -> anyhow::Result<()> {
        let e = self.notes.entry(pitch.clone()).or_default();
        let (turn_on, number) = if velocity == 0 {
            if *e == 0 {
                log::warn!("csound received note off for unknown note {pitch}");
                return Ok(());
            }
            *e -= 1;
            if *e > 0 {
                // The note is on more than once
                return Ok(());
            }
            let Some(n) = self.note_to_number.get(pitch) else {
                log::warn!("no note number known for note {pitch}");
                return Ok(());
            };
            (false, *n)
        } else {
            *e += 1;
            if *e > 1 {
                // The note is already on
                return Ok(());
            }
            // Pick a note number
            let mut n = 1;
            loop {
                // Find first unused note number >= 1
                if !self.number_to_note.contains_key(&n) {
                    break;
                }
                n += 1;
            }
            (true, n)
        };

        let message = if turn_on {
            self.note_to_number.insert(pitch.clone(), number);
            self.number_to_note.insert(number, pitch.clone());
            let freq = pitch.as_float();
            format!("i 1.{number} 0 -1 {freq}")
        } else {
            self.note_to_number.remove(pitch);
            self.number_to_note.remove(&number);
            format!("i 1.{number} 0 0")
        };

        let num_notes = self.note_to_number.len();
        let amp = if num_notes == 0 {
            0.0
        } else {
            // TODO: figure out a good formula for this. This is a good start.
            0.25 / (num_notes as f32).sqrt()
        };
        self.api
            .input_message(format!(r#"i "SetChan" 0 -1 {amp:4} "amp""#))
            .await?;
        self.api.input_message(message).await?;
        Ok(())
    }
}

pub async fn run_csound(
    mut events_rx: events::Receiver,
    events_tx: events::WeakSender,
) -> anyhow::Result<()> {
    let mut csound = CSound::new(events_tx).await?;
    csound
        .api
        .input_message("i \"SetChan\" 0 -1 0.7 \"amp\"")
        .await?;
    while let Some(event) = events::receive_check_lag(&mut events_rx, Some("csound player")).await {
        let Event::PlayNote(e) = event else {
            continue;
        };
        csound.handle_note(&e.pitch, e.velocity).await?;
    }
    csound.api.shutdown().await;
    Ok(())
}
