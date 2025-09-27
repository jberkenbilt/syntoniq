use clap::Parser;
use std::fs;
use syntoniq_common::parsing::pass3;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    filename: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let data = fs::read(&cli.filename)?;
    let input = str::from_utf8(&data)?;
    let r = pass3::parse3(input);
    match r {
        Err(diags) => anstream::eprintln!("{}", diags.render(&cli.filename, input)),
        Ok(_) => {
            // TODO
        }
    }
    Ok(())
}
