use anstream::stream::AsLockedWrite;
use clap::Parser;
use std::{fs, io};
use syntoniq_common::parsing::pass3;
use syntoniq_common::parsing::score::{Directive, FromRawDirective};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    filename: String,
    #[arg(long)]
    show_help: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if cli.show_help {
        Directive::show_help(&mut io::stdout().as_locked_write())?;
        return Ok(());
    }
    let data = fs::read(&cli.filename)?;
    let input = str::from_utf8(&data)?;
    let r = pass3::parse3(input);
    match r {
        Err(diags) => anstream::eprintln!("{}", diags.render(&cli.filename, input)),
        Ok(timeline) => {
            serde_json::to_writer_pretty(io::stdout().as_locked_write(), &timeline)?;
        }
    }
    Ok(())
}
