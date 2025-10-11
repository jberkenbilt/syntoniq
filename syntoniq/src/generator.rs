use clap::Parser;
use std::path::PathBuf;
mod midi;

#[derive(Parser)]
pub struct GenerateOptions {
    #[arg(long)]
    score: PathBuf,
    #[arg(long)]
    /// Output MIDI file. This can be played with Timidity++ or processed with other software that
    /// properly handles MTS (Midi Tuning System) messages.
    midi: Option<PathBuf>,
}

pub fn run(options: GenerateOptions) -> anyhow::Result<()> {
    let timeline = crate::parse(&options.score)?;
    println!("syntoniq score '{}' is valid", options.score.display());
    if let Some(midi_file) = options.midi {
        midi::generate(&timeline, midi_file)?;
    }
    Ok(())
}
