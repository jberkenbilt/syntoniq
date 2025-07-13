use clap::{Command, CommandFactory};
use clap::{Parser, Subcommand};
use clap_complete::{Generator, Shell, aot};
use log::LevelFilter;
use qlaunchpad::controller::{Controller, ToDevice};
use std::error::Error;
use std::{env, io};

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
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

    // Create midi controller.
    let controller = Controller::new(port.to_string()).await?;

    // Make sure everything is cleaned up on exit.
    let sender = controller.sender();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        let _ = sender.send(ToDevice::Shutdown).await;
    });

    match cli.command {
        Commands::Events => events_main(controller).await,
        Commands::Completion { .. } => unreachable!("already handled"),
    }
}

async fn events_main(controller: Controller) -> Result<(), Box<dyn Error + Sync + Send>> {
    let mut rx = controller.subscribe();
    let h = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            println!("{event}");
        }
    });
    let sender = controller.sender();
    sender.send(ToDevice::Data(vec![0x90, 59, 0x2d])).await?;
    log::info!("Hit CTRL-C to exit");
    h.await?;
    controller.join().await
}
