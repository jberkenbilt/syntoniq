use crate::events;
use crate::events::{Event, KeyEvent};
use midir::MidiOutput;
use midir::os::unix::VirtualOutput;
use std::error::Error;

pub async fn play_midi(
    mut events_rx: events::Receiver,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let (tx, rx) = flume::unbounded();
    let h = tokio::spawn(async move {
        while let Ok(event) = events_rx.recv().await {
            let Event::Key(key_event) = event else {
                continue;
            };
            tx.send_async(key_event).await.unwrap();
        }
    });

    let midi_out = MidiOutput::new("q-launchpad")?;
    let mut output_connection = midi_out
        .create_virtual("QLaunchPad")
        .map_err(crate::to_sync_send)?;
    tokio::task::spawn_blocking(move || -> Result<(), Box<dyn Error + Sync + Send>> {
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
