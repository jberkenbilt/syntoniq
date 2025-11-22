use crate::generator::midi::MidiStyle;
use anyhow::bail;
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use syntoniq_common::parsing;
use syntoniq_common::parsing::Timeline;

mod csound;
mod midi;

pub const CSOUND_TEMPLATE: &str = csound::DEFAULT_TEMPLATE;

#[derive(Parser)]
pub struct GenerateOptions {
    #[arg(long)]
    score: PathBuf,
    /// Output a JSON dump of the timeline.
    #[arg(long)]
    json: Option<PathBuf>,
    /// Output MIDI file with MTS data. This can be played with Timidity++ or processed with other
    /// software that properly handles MTS (Midi Tuning System) messages embedded in MIDI files.
    #[arg(long)]
    midi_mts: Option<PathBuf>,
    /// Output MIDI file with MPE data. This can be played with Surge-XT and will load properly
    /// into most Digital Audio Workstations. As of 2025, Timidity++ does not follow pitch-bend
    /// dta.
    #[arg(long)]
    midi_mpe: Option<PathBuf>,
    /// Output CSound file. Use the `--csound-template` option to use a template other than the
    /// built-in one.
    #[arg(long)]
    csound: Option<PathBuf>,
    /// Override the built-in CSound template. The template has to conform to a certain structure
    /// to be usable. Run `syntoniq csound-template` to print the contents of the built-in template.
    /// You can also use a previous output as a template to just replace the generated portion.
    #[arg(long)]
    csound_template: Option<PathBuf>,
    #[command(flatten)]
    parse_options: parsing::Options,
}

fn generate_json(timeline: &Timeline, json_file: PathBuf) -> anyhow::Result<()> {
    fs::write(&json_file, serde_json::to_string_pretty(&timeline)? + "\n")?;
    println!("JSON output written to {}", json_file.display());
    Ok(())
}

pub fn run(options: GenerateOptions) -> anyhow::Result<()> {
    let data = fs::read(&options.score)?;
    let score_file = options.score.display();
    let src = str::from_utf8(&data)?;
    let timeline = parsing::timeline(
        &options.score.display().to_string(),
        src,
        &options.parse_options,
    )?;
    println!("syntoniq score '{}' is valid", options.score.display());
    let mut errors = Vec::new();
    if let Some(json_file) = options.json
        && let Err(e) = generate_json(&timeline, json_file)
    {
        errors.push(format!("{score_file} -> JSON: {e}"));
    }
    if let Some(midi_file) = options.midi_mts
        && let Err(e) = midi::generate(&timeline, midi_file, MidiStyle::Mts)
    {
        errors.push(format!("{score_file} -> MIDI (MTS): {e}"));
    }
    if let Some(midi_file) = options.midi_mpe
        && let Err(e) = midi::generate(&timeline, midi_file, MidiStyle::Mpe)
    {
        errors.push(format!("{score_file} -> MIDI (MTS): {e}"));
    }
    if let Some(csound_file) = options.csound
        && let Err(e) = csound::generate(&timeline, csound_file, options.csound_template)
    {
        errors.push(format!("{score_file} -> CSound: {e}"));
    }
    if !errors.is_empty() {
        bail!("{}", errors.join("\n"))
    }
    Ok(())
}

#[cfg(test)]
mod tests;
