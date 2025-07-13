use clap::{Command, CommandFactory};
use clap::{Parser, Subcommand};
use clap_complete::{Generator, Shell, aot};
use log::LevelFilter;
use qlaunchpad::controller::{Controller, ToDevice};
use std::error::Error;
use std::{env, io, thread};

/// This command operates with a Launchpad MK3 Pro MIDI Controller in various ways.
/// Logging is controlled with RUST_LOG; see docs for the env_logger crate.
/// If RUST_LOG is not set, the log level defaults to Info.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Substring to match for midi port; run amidi -l
    #[arg(long)]
    port: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Log device events during interaction
    Events,
    /// Generate shell completion
    Completion {
        /// shell
        shell: Shell,
    },
}

fn print_completions<G: Generator>(generator: G, cmd: &mut Command) {
    aot::generate(
        generator,
        cmd,
        cmd.get_name().to_string(),
        &mut io::stdout(),
    );
}

fn to_sync_send(e: Box<dyn Error>) -> Box<dyn Error + Sync + Send> {
    e.to_string().into()
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let cli = Cli::parse();
    if let Commands::Completion { shell } = cli.command {
        let mut cmd = Cli::command();
        print_completions(shell, &mut cmd);
        return Ok(());
    }
    let Some(port) = cli.port else {
        eprintln!("The --port option is required");
        std::process::exit(2);
    };

    let mut log_builder = env_logger::builder();
    if env::var("RUST_LOG").is_err() {
        log_builder.filter_level(LevelFilter::Info);
    }
    log_builder.init();
    match cli.command {
        Commands::Events => events_main(&port),
        Commands::Completion { shell } => {
            let mut cmd = Cli::command();
            print_completions(shell, &mut cmd);
            Ok(())
        }
    }
}

fn events_main(port: &str) -> Result<(), Box<dyn Error + Sync + Send>> {
    let mut c = Controller::new(port).map_err(to_sync_send)?;
    let sender = c.sender();
    ctrlc::set_handler(move || {
        let _ = sender.send(ToDevice::Shutdown);
    })?;
    let sender = c.sender();
    let th = thread::spawn(move || c.run().map_err(to_sync_send));
    sender.send(ToDevice::Data(vec![0x90, 59, 0x2d]))?;
    log::info!("Hit CTRL-C to exit");
    th.join().unwrap()
}
