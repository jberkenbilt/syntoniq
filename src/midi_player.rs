use crate::events::{Event, KeyEvent};
use crate::{events, to_anyhow};
use midir::MidiOutput;
use midir::os::unix::VirtualOutput;

pub async fn play_midi(mut events_rx: events::Receiver) -> anyhow::Result<()> {
    let (tx, rx) = flume::unbounded();
    let h = tokio::spawn(async move {
        while let Some(event) = events::receive_check_lag(&mut events_rx, Some("midi player")).await
        {
            let Event::Key(key_event) = event else {
                continue;
            };
            tx.send_async(key_event).await.unwrap();
        }
    });

    let midi_out = MidiOutput::new("q-launchpad")?;
    let mut output_connection = midi_out.create_virtual("QLaunchPad").map_err(to_anyhow)?;
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        // TODO: when we have it, this should subscribe to Note events, not Key events.
        while let Ok(KeyEvent { key, velocity }) = rx.recv() {
            output_connection.send(&[0x90, 100 - key, velocity])?;
        }
        Ok(())
    })
    .await
    .unwrap()?;
    h.await?;
    Ok(())
}
