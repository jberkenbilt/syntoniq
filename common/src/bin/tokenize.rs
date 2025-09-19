use clap::Parser;
use serde_json::json;
use std::io::Write;
use std::{fs, io};
use syntoniq_common::parsing::{pass1, pass2};

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
    if cli.json {
        let mut results = Vec::new();
        let r = pass1::parse1(input);
        results.push(json!(&r));
        if r.is_ok() {
            let r = pass2::parse2(input);
            results.push(json!(&r));
        }
        serde_json::to_writer_pretty(io::stdout(), &results)?;
        _ = io::stdout().write(b"\n");
    } else {
        let r = pass2::parse2(input);
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
