use clap::Parser;
use std::fs;
use std::path::PathBuf;
mod midi;

#[derive(Parser)]
pub struct GenerateOptions {
    #[arg(long)]
    score: PathBuf,
    /// Output MIDI file. This can be played with Timidity++ or processed with other software that
    /// properly handles MTS (Midi Tuning System) messages.
    #[arg(long)]
    midi: Option<PathBuf>,
}

pub fn run(options: GenerateOptions) -> anyhow::Result<()> {
    let data = fs::read(&options.score)?;
    let src = str::from_utf8(&data)?;
    let timeline = crate::parse(&options.score.display().to_string(), src)?;
    println!("syntoniq score '{}' is valid", options.score.display());
    if let Some(midi_file) = options.midi {
        midi::generate(&timeline, midi_file)?;
    }
    Ok(())
}
