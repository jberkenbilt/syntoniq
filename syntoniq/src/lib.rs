use std::fs;
use std::path::Path;
use syntoniq_common::parsing;
use syntoniq_common::parsing::Timeline;
pub mod generator;

pub fn parse(file: impl AsRef<Path>) -> anyhow::Result<Timeline> {
    let data = fs::read(&file)?;
    let src = str::from_utf8(&data)?;
    match parsing::parse(src) {
        Ok(timeline) => Ok(timeline),
        Err(diags) => {
            anstream::eprintln!(
                "{}",
                diags.render(&file.as_ref().display().to_string(), src)
            );
            std::process::exit(2);
        }
    }
}
