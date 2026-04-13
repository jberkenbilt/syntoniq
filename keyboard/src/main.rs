use anyhow::bail;
use clap::CommandFactory;
use clap::{Parser, Subcommand};
use clap_complete::Shell;
use log::LevelFilter;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use syntoniq_kbd::controller::Controller;
use syntoniq_kbd::engine::{Keyboard, SoundType};
use syntoniq_kbd::events::Events;
use syntoniq_kbd::hexboard::HexBoard;
use syntoniq_kbd::launchpad::Launchpad;
use syntoniq_kbd::view::web;
use syntoniq_kbd::{DeviceType, prompt};
use syntoniq_kbd::{csound, engine};
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
    /// Interactive command-line prompt-based note/chord player
    Prompt(Prompt),
    /// Output the built-in keyboard configuration
    DefaultConfig,
    /// Output the built-in Csound text file containing the instrument
    CsoundText,
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
    #[clap(flatten)]
    sound_config: SoundConfig,
}

#[derive(Parser)]
struct Prompt {
    #[clap(flatten)]
    sound_config: SoundConfig,
}

#[derive(Parser)]
struct SoundConfig {
    /// Send notes to a virtual output port named Syntoniq
    #[arg(long)]
    midi: bool,
    /// Additional option to pass to csound, e.g. --csound-arg=-odac1; repeatable
    #[arg(long)]
    csound_arg: Vec<String>,
    /// Csound file containing the keyboard's instrument; start with `syntoniq-kbd csound-text`
    /// and modify based on the comments.
    #[arg(long)]
    csound_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut cli = Cli::parse();
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
        Commands::CsoundText => {
            print!("{}", csound::CSOUND_TEXT);
            return Ok(());
        }
        Commands::Run { .. } | Commands::Prompt { .. } => {}
    }

    let mut log_builder = env_logger::builder();
    if env::var("RUST_LOG").is_err() {
        log_builder.filter_level(LevelFilter::Info);
    }
    log_builder.init();

    let events = Events::new();
    let events_tx = events.sender().await;
    let events_rx = events.receiver();

    let sound_config = match &mut cli.command {
        Commands::Run(r) => &mut r.sound_config,
        Commands::Prompt(r) => &mut r.sound_config,
        _ => unreachable!(),
    };

    let sound_type = if sound_config.midi {
        SoundType::Midi
    } else {
        #[cfg(feature = "csound")]
        {
            SoundType::Csound {
                file: sound_config.csound_file.take(),
                args: std::mem::take(&mut sound_config.csound_arg),
            }
        }
        #[cfg(not(feature = "csound"))]
        {
            bail!("MIDI not requested and csound not available");
        }
    };
    engine::start_sound(sound_type, events_tx.clone(), events_rx.resubscribe()).await;

    let main_handle = match cli.command {
        Commands::Run(run) => {
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
            let h =
                engine::start_keyboard(Some(controller), keyboard.clone(), events_rx.resubscribe())
                    .await?;
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
            engine::run(
                run.score,
                keyboard,
                events_tx.clone(),
                events_rx.resubscribe(),
            )
            .await?;
            h
        }
        Commands::Prompt(_) => prompt::run(events),
        Commands::DefaultConfig | Commands::CsoundText | Commands::Completion { .. } => {
            unreachable!("already handled")
        }
    };

    drop(events_tx);
    drop(events_rx);
    main_handle.await?
}
