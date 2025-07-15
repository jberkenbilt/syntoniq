use crate::controller::{Controller, FromDevice};
use midir::MidiOutput;
use midir::os::unix::VirtualOutput;
use std::error::Error;

pub async fn play_midi(controller: &mut Controller) -> Result<(), Box<dyn Error + Sync + Send>> {
    //TODO: This is subscribing directly from the controller, but instead, we might want something
    // else to subscribe and translate these to some kind of note event. Then other things, like
    // a csound output module and this, can subscribe to *that* instead. That might be a cleaner
    // way to keep the button to note mapping in one place.
    let mut controller_rx = controller.receiver();
    let (tx, rx) = flume::unbounded::<FromDevice>();
    let h = tokio::spawn(async move {
        while let Ok(event) = controller_rx.recv().await {
            tx.send_async(event).await.unwrap();
        }
    });

    let midi_out = MidiOutput::new("q-launchpad")?;
    let mut output_connection = midi_out
        .create_virtual("QLaunchPad")
        .map_err(crate::to_sync_send)?;
    tokio::task::spawn_blocking(move || -> Result<(), Box<dyn Error + Sync + Send>> {
        while let Ok(event) = rx.recv() {
            let FromDevice::Key { key, velocity } = event else {
                continue;
            };
            // TODO: need key to note mapping; should use same logic as key to pitch mapping
            output_connection.send(&[0x90, 100 - key, velocity])?;
        }
        Ok(())
    })
    .await
    .unwrap()?;
    h.await?;
    Ok(())
}
