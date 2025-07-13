use clap::{Command, CommandFactory};
use clap::{Parser, Subcommand};
use clap_complete::{Generator, Shell, aot};
use log::LevelFilter;
use qlaunchpad::controller::{Controller, LightMode, ToDevice};
use std::error::Error;
use std::path::PathBuf;
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
    /// Experiment with LED colors. When you touch a square, it flashes.
    /// To swap two squares, touch the first square (it flashes), then the second.
    /// To set a color, touch a square twice (it pulses), then use the bottom
    /// controllers to set the color. Each controller represents a digit from 0 to f
    /// in left-to-right, top-to-bottom order.
    /// Use the `<` and `>` keys to shift between pages.
    Colors {
        /// A file containing rows of at most 8 space-separated, two-digit hex codes per line
        /// representing the colors. If not specified, the layout is 00 through 7F over two pages.
        #[arg(long)]
        initial: Option<PathBuf>,
    },
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
    let mut controller = Controller::new(port.to_string()).await?;

    // Make sure everything is cleaned up on exit.
    let sender = controller.sender();
    tokio::spawn(async move {
        log::info!("Hit CTRL-C to exit");
        let _ = tokio::signal::ctrl_c().await;
        let _ = sender.send(ToDevice::Shutdown).await;
    });

    match cli.command {
        Commands::Completion { .. } => unreachable!("already handled"),
        Commands::Events => events_main(&mut controller).await,
        Commands::Colors { initial } => colors_main(&mut controller, initial).await,
    }?;
    controller.join().await
}

async fn events_main(controller: &mut Controller) -> Result<(), Box<dyn Error + Sync + Send>> {
    let mut rx = controller.subscribe();
    while let Ok(event) = rx.recv().await {
        println!("{event}");
    }
    Ok(())
}

async fn colors_main(
    controller: &mut Controller,
    _initial: Option<PathBuf>,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let mut rx = controller.subscribe();
    let sender = controller.sender();
    for position in 1..=108 {
        sender.send(ToDevice::LightOff { position }).await.unwrap();
    }
    sender
        .send(ToDevice::LightOn {
            mode: LightMode::On,
            position: 70,
            color: 3,
        })
        .await
        .unwrap();
    sender
        .send(ToDevice::LightOn {
            mode: LightMode::On,
            position: 80,
            color: 3,
        })
        .await
        .unwrap();
    let mut color = 0;
    for row in (1..=8).rev() {
        for col in 1..=8 {
            let position: u8 = 10 * row + col;
            sender
                .send(ToDevice::LightOn {
                    mode: LightMode::On,
                    position,
                    color,
                })
                .await
                .unwrap();
            color += 1;
        }
    }
    while let Ok(event) = rx.recv().await {
        println!("{event}");
    }
    Ok(())
}
