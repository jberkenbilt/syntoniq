use clap::Parser;
use std::{fs, io};
use syntoniq_common::parsing::pass2;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Tokenize an input file, mainly for debugging the parser.
/// Set CLICOLOR_FORCE to force color output; set NO_COLOR to force non-color output.
struct Cli {
    #[arg(long)]
    /// Show JSON output instead of pretty-printed text
    json: bool,
    filename: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let data = fs::read(&cli.filename)?;
    let input = str::from_utf8(&data)?;
    let r = pass2::parse2(input);
    if cli.json {
        serde_json::to_writer_pretty(io::stdout(), &r)?;
    } else {
        match r {
            Err(diags) => anstream::eprintln!("{}", diags.render(&cli.filename, input)),
            Ok(tokens) => {
                for t in tokens {
                    println!("{t}")
                }
            }
        }
    }
    Ok(())
}
