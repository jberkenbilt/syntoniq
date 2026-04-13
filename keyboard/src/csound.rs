use crate::csound::wrapper::CsoundApi;
use crate::events;
use crate::events::Event;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syntoniq_common::pitch::Pitch;

mod wrapper;

struct Csound {
    api: CsoundApi,
    notes: HashMap<Pitch, u32>,
    note_to_number: HashMap<Pitch, u32>,
    number_to_note: HashMap<u32, Pitch>,
}

pub const CSOUND_TEXT: &str = include_str!("sound.csd");

impl Csound {
    pub async fn new(
        file: Option<impl AsRef<Path>>,
        events_tx: events::WeakSender,
        args: Vec<String>,
    ) -> anyhow::Result<Self> {
        let csound_text = match file {
            None => Cow::Borrowed(CSOUND_TEXT),
            Some(p) => {
                let buf = fs::read(p)?;
                let text = String::from_utf8(buf)?;
                Cow::Owned(text)
            }
        };
        let api = CsoundApi::new(&csound_text, events_tx, args).await?;
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
            format!("i -1.{number} 0 0")
        };

        self.api.input_message(message).await?;
        Ok(())
    }
}

pub async fn run_csound(
    csound_file: Option<impl AsRef<Path>>,
    mut events_rx: events::Receiver,
    events_tx: events::WeakSender,
    args: Vec<String>,
) -> anyhow::Result<()> {
    let mut csound = Csound::new(csound_file, events_tx, args).await?;
    while let Some(event) = events::receive_check_lag(&mut events_rx, Some("csound player")).await {
        let Event::PlayNote(e) = event else {
            continue;
        };
        csound.handle_note(&e.pitch, e.velocity).await?;
    }
    csound.api.shutdown().await;
    Ok(())
}
