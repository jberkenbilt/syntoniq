use clap::Parser;
use serde_json::json;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};
use syntoniq_common::parsing::{Options, pass1, pass2, pass3};

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Tokenize an input file, mainly for debugging the parser.
/// Set CLICOLOR_FORCE to force color output; set NO_COLOR to force non-color output.
struct Cli {
    /// Show JSON output instead of pretty-printed text
    #[arg(long)]
    json: bool,
    #[arg(long)]
    /// Take parser options as json
    parser_options: Option<PathBuf>,
    filename: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let data = fs::read(&cli.filename)?;
    let input = str::from_utf8(&data)?;
    let options = match cli.parser_options {
        None => Options::default(),
        Some(p) => serde_json::from_slice::<Options>(&fs::read(p)?)?,
    };
    if cli.json {
        let mut results = Vec::new();
        let r = pass1::parse1(input);
        results.push(json!(&r));
        if r.is_ok() {
            let r = pass2::parse2(input);
            results.push(json!(&r));
            if r.is_ok() {
                let r = pass3::parse3(input, &options);
                results.push(json!(&r));
            }
        };
        serde_json::to_writer_pretty(io::stdout(), &results)?;
        _ = io::stdout().write(b"\n");
    } else {
        match pass3::parse3(input, &options) {
            Err(diags) => anstream::eprintln!("{}", diags.render(&cli.filename, input)),
            Ok(_) => {
                println!("file is valid");
            }
        }
    }
    Ok(())
}
