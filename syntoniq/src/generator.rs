use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct GenerateOptions {
    #[arg(long)]
    score: PathBuf,
}

pub fn run(options: GenerateOptions) -> anyhow::Result<()> {
    let _timeline = crate::parse(&options.score)?;
    Ok(())
}
