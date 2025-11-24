#[cfg(not(feature = "csound"))]
use anyhow::bail;
use clap::CommandFactory;
use clap::{Parser, Subcommand};
use clap_complete::Shell;
use log::LevelFilter;
use std::env;
use std::sync::Arc;
use syntoniq_kbd::engine::{Keyboard, SoundType};
use syntoniq_kbd::events::Events;
use syntoniq_kbd::launchpad::Launchpad;
use syntoniq_kbd::view::web;
use syntoniq_kbd::{engine, events};

/// This command operates with a Launchpad MK3 Pro MIDI Controller in various ways.
/// Logging is controlled with RUST_LOG; see docs for the env_logger crate.
/// If RUST_LOG is not set, the log level defaults to Info.
/// Set RUST_LOG=syntoniq_kbd::module::path=level to see messages for a given module.
/// Set RUST_LOG=syntoniq_kbd to see all messages.
#[derive(Parser)]
#[command(version, about, long_about = None, verbatim_doc_comment)]
struct Cli {
    /// Substring to match for midi port; run amidi -l
    #[arg(long)]
    port: Option<String>,

    #[arg(long)]
    no_dev: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Main command -- handle events and send music commands
    Run {
        /// Syntoniq score file containing layouts
        #[arg(long)]
        score: String,
        /// Send notes to a virtual output port named Syntoniq
        #[arg(long)]
        midi: bool,
    },
    /// Log device events during interaction
    Events,
    /// Generate shell completion
    Completion {
        /// shell
        shell: Shell,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if let Commands::Completion { shell } = cli.command {
        let mut cmd = Cli::command();
        syntoniq_common::cli_completions(shell, &mut cmd);
        return Ok(());
    }
    if cli.port.is_none() && !cli.no_dev {
        eprintln!("One of --port or --no-dev is required");
        std::process::exit(2);
    };

    let mut log_builder = env_logger::builder();
    if env::var("RUST_LOG").is_err() {
        log_builder.filter_level(LevelFilter::Info);
    }
    log_builder.init();

    let events = Events::new();
    let events_tx = events.sender().await;
    let events_rx = events.receiver();

    // Create midi controller.
    let tx2 = events_tx.clone();
    let mut rx2 = events_rx.resubscribe();
    let mut keyboard: Option<Arc<dyn Keyboard>> = None;
    let main_handle = match cli.port {
        Some(port) => {
            let lp = Arc::new(Launchpad::new(tx2));
            keyboard = Some(lp.clone());
            lp.run(port.to_string(), rx2).await?
        }
        None => tokio::spawn(async move {
            while events::receive_check_lag(&mut rx2, None).await.is_some() {}
            Ok(())
        }),
    };

    let tx2 = events_tx.clone();
    let rx2 = events_rx.resubscribe();
    tokio::spawn(async move {
        web::http_view(tx2, rx2, 8440).await;
    });

    // Make sure everything is cleaned up on exit.
    tokio::spawn(async move {
        log::info!("Hit CTRL-C to exit");
        let _ = tokio::signal::ctrl_c().await;
        events.shutdown().await;
    });

    match cli.command {
        Commands::Completion { .. } => unreachable!("already handled"),
        Commands::Events => events_main(events_rx.resubscribe()).await,
        Commands::Run { score, midi } => {
            let sound_type = if midi {
                SoundType::Midi
            } else {
                #[cfg(feature = "csound")]
                {
                    SoundType::Csound
                }
                #[cfg(not(feature = "csound"))]
                bail!("MIDI not requested and csound not available");
            };
            // TODO: keyboard
            engine::run(
                &score,
                sound_type,
                keyboard.unwrap(), // TODO: is no-device dead?
                events_tx.clone(),
                events_rx.resubscribe(),
            )
            .await
        }
    }?;
    drop(events_tx);
    drop(events_rx);
    main_handle.await?
}

async fn events_main(mut rx: events::Receiver) -> anyhow::Result<()> {
    while let Ok(event) = rx.recv().await {
        println!("{event:?}");
    }
    Ok(())
}
