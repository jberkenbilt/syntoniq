use anyhow::bail;
use clap::CommandFactory;
use clap::{Parser, Subcommand};
use clap_complete::Shell;
use log::LevelFilter;
use std::env;
use std::sync::Arc;
use syntoniq_kbd::DeviceType;
use syntoniq_kbd::controller::Controller;
use syntoniq_kbd::engine;
use syntoniq_kbd::engine::{Keyboard, SoundType};
use syntoniq_kbd::events::Events;
use syntoniq_kbd::hexboard::HexBoard;
use syntoniq_kbd::launchpad::Launchpad;
use syntoniq_kbd::view::web;
use tokio::sync::oneshot;

/// This command operates with a Launchpad MK3 Pro MIDI Controller in various ways.
/// Logging is controlled with RUST_LOG; see docs for the env_logger crate.
/// If RUST_LOG is not set, the log level defaults to Info.
/// Set RUST_LOG=syntoniq_kbd::module::path=level to see messages for a given module.
/// Set RUST_LOG=syntoniq_kbd to see all messages.
#[derive(Parser)]
#[command(version, about, long_about = None, verbatim_doc_comment)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Main command -- handle events and send music commands
    Run(Run),
    /// Output the built-in keyboard configuration
    DefaultConfig,
    /// Generate shell completion
    Completion {
        /// shell
        shell: Shell,
    },
}

#[derive(Parser)]
struct Run {
    /// Substring to match for midi port; run amidi -l
    #[arg(long)]
    port: String,
    /// Syntoniq score file containing layouts; if omitted, a built-in default is used.
    #[arg(long)]
    score: Option<String>,
    /// Send notes to a virtual output port named Syntoniq
    #[arg(long)]
    midi: bool,
    /// Additional option to pass to csound, e.g. --csound-arg=-odac1; repeatable
    #[arg(long)]
    csound_arg: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Completion { shell } => {
            let mut cmd = Cli::command();
            syntoniq_common::cli_completions(shell, &mut cmd);
            return Ok(());
        }
        Commands::DefaultConfig => {
            print!("{}", engine::DEFAULT_SCORE);
            return Ok(());
        }
        Commands::Run { .. } => {}
    }

    let mut log_builder = env_logger::builder();
    if env::var("RUST_LOG").is_err() {
        log_builder.filter_level(LevelFilter::Info);
    }
    log_builder.init();

    let events = Events::new();
    let events_tx = events.sender().await;
    let events_rx = events.receiver();

    // Create midi controller.
    let Commands::Run(run) = cli.command else {
        unreachable!("already handled");
    };
    let tx2 = events_tx.clone();
    let (id_tx, id_rx) = oneshot::channel();
    let controller = Controller::new(&run.port, id_tx)?;
    let device_type = id_rx.await?;
    let keyboard = match device_type {
        DeviceType::Empty => {
            bail!("unable to identify device on port {}", run.port);
        }
        DeviceType::Launchpad => Arc::new(Launchpad::new(tx2)) as Arc<dyn Keyboard>,
        DeviceType::HexBoard => Arc::new(HexBoard::new(tx2)) as Arc<dyn Keyboard>,
    };
    let main_handle =
        engine::start_keyboard(Some(controller), keyboard.clone(), events_rx.resubscribe()).await?;
    let tx2 = events_tx.clone();
    let rx2 = events_rx.resubscribe();
    tokio::spawn(async move {
        web::http_view(tx2, rx2, 8440, device_type).await;
    });

    // Make sure everything is cleaned up on exit.
    tokio::spawn(async move {
        println!("Hit CTRL-C to exit");
        let _ = tokio::signal::ctrl_c().await;
        events.shutdown().await;
    });

    let sound_type = if run.midi {
        SoundType::Midi
    } else {
        #[cfg(feature = "csound")]
        {
            SoundType::Csound(run.csound_arg)
        }
        #[cfg(not(feature = "csound"))]
        {
            let _ = run.csound_arg;
            bail!("MIDI not requested and csound not available");
        }
    };
    engine::run(
        run.score,
        sound_type,
        keyboard,
        events_tx.clone(),
        events_rx.resubscribe(),
    )
    .await?;
    drop(events_tx);
    drop(events_rx);
    main_handle.await?
}
