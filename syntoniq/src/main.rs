use clap::CommandFactory;
use clap::{Parser, Subcommand};
use clap_complete::Shell;
use log::LevelFilter;
use std::{env, process};
use syntoniq::generator;
use syntoniq::generator::GenerateOptions;
use syntoniq_common::parsing;

/// Logging is controlled with RUST_LOG; see docs for the env_logger crate.
/// If RUST_LOG is not set, the log level defaults to Info.
/// Set RUST_LOG=syntoniq::module::path=level to see messages for a given module.
/// Set RUST_LOG=syntoniq to see all messages.
#[derive(Parser)]
#[command(version, about, long_about = None, verbatim_doc_comment)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Csound and/or MIDI output. If no output is specified, this just parses the score
    /// and reports errors, if any.
    Generate(GenerateOptions),
    /// Show built-in documentation
    Doc,
    /// Generate shell completion
    Completion {
        /// shell
        shell: Shell,
    },
    /// Write built-in CSound template to standard output
    CsoundTemplate,
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut log_builder = env_logger::builder();
    if env::var("RUST_LOG").is_err() {
        log_builder.filter_level(LevelFilter::Info);
    }
    log_builder.init();

    match cli.command {
        Commands::Completion { shell } => {
            let mut cmd = Cli::command();
            syntoniq_common::cli_completions(shell, &mut cmd);
            Ok(())
        }
        Commands::CsoundTemplate => {
            print!("{}", generator::CSOUND_TEMPLATE);
            Ok(())
        }
        Commands::Generate(options) => generator::run(options),
        Commands::Doc => parsing::show_help(),
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        eprintln!("run 'syntoniq doc' for built-in documentation");
        process::exit(2);
    }
}
