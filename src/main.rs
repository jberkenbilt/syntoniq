use clap::{Command, CommandFactory};
use clap::{Parser, Subcommand};
use clap_complete::{Generator, Shell, aot};
use log::LevelFilter;
use qlaunchpad::controller::Controller;
use qlaunchpad::events::{Color, Event, Events, KeyEvent, LightEvent, LightMode};
use qlaunchpad::view::web;
use qlaunchpad::{controller, engine, events};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, io};

// TODO: format or wrap help text

/// This command operates with a Launchpad MK3 Pro MIDI Controller in various ways.
/// Logging is controlled with RUST_LOG; see docs for the env_logger crate.
/// If RUST_LOG is not set, the log level defaults to Info.
/// Set RUST_LOG=qlaunchpad::module::path=level to see messages for a given module.
/// Set RUST_LOG=qlaunchpad to see all messages.
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
    /// Main command -- handle events and send music commands
    Run {
        /// toml-format config file
        #[arg(long)]
        config_file: PathBuf,
        /// Send notes to a virtual output port named QLaunchPad
        #[arg(long)]
        midi: bool,
    },
    /// Log device events during interaction
    Events,
    /// Display color choices
    Colors,
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
async fn main() -> anyhow::Result<()> {
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

    let events = Events::new();
    let events_tx = events.sender();
    let events_rx = events.receiver();

    // Create midi controller.
    let controller =
        Controller::new(port.to_string(), events_tx.clone(), events_rx.resubscribe()).await?;
    let rx2 = events_rx.resubscribe();
    tokio::spawn(async move {
        web::http_view(rx2, 8440).await;
    });

    // Make sure everything is cleaned up on exit.
    tokio::spawn(async move {
        log::info!("Hit CTRL-C to exit");
        let _ = tokio::signal::ctrl_c().await;
        events.shutdown();
    });

    match cli.command {
        Commands::Completion { .. } => unreachable!("already handled"),
        Commands::Events => events_main(events_rx.resubscribe()).await,
        Commands::Colors => colors_main(events_tx.clone(), events_rx.resubscribe()).await,
        Commands::Run { config_file, midi } => {
            engine::run(
                config_file,
                midi,
                events_tx.clone(),
                events_rx.resubscribe(),
            )
            .await
        }
    }?;
    drop(events_tx);
    drop(events_rx);
    controller.join().await
}

async fn events_main(mut rx: events::Receiver) -> anyhow::Result<()> {
    while let Ok(event) = rx.recv().await {
        println!("{event}");
    }
    Ok(())
}

async fn colors_main(
    events_tx: events::Sender,
    mut events_rx: events::Receiver,
) -> anyhow::Result<()> {
    let Some(tx) = events_tx.upgrade() else {
        return Ok(());
    };
    controller::clear_lights(&tx).await?;
    // Light all control keys
    for range in [1..=8, 101..=108, 90..=99] {
        for position in range {
            tx.send(Event::Light(LightEvent {
                mode: LightMode::On,
                position,
                color: Color::Active,
                label1: String::new(),
                label2: String::new(),
            }))?;
        }
    }
    for row in 1..=8 {
        for position in [row * 10, row * 10 + 9] {
            tx.send(Event::Light(LightEvent {
                mode: LightMode::On,
                position,
                color: Color::Active,
                label1: String::new(),
                label2: String::new(),
            }))?;
        }
    }
    for (position, color) in [
        (11, Color::FifthOff),
        (12, Color::MajorThirdOff),
        (13, Color::MinorThirdOff),
        (14, Color::TonicOff),
        (15, Color::FifthOn),
        (16, Color::MajorThirdOn),
        (17, Color::MinorThirdOn),
        (18, Color::TonicOn),
    ] {
        tx.send(Event::Light(LightEvent {
            mode: LightMode::On,
            position,
            color,
            label1: String::new(),
            label2: String::new(),
        }))?;
    }
    let simulated = [
        (Color::TonicOff, Color::TonicOn, [32, 51]),
        (Color::SingleStepOff, Color::SingleStepOn, [33, 52]),
        (Color::MinorThirdOff, Color::MinorThirdOn, [43, 62]),
        (Color::MajorThirdOff, Color::MajorThirdOn, [34, 53]),
        (Color::FifthOff, Color::FifthOn, [44, 63]),
        (Color::FifthOff, Color::FifthOn, [45, 64]),
        (Color::MinorThirdOff, Color::MinorThirdOn, [46, 65]),
        (Color::OtherOff, Color::OtherOn, [47, 66]),
        (Color::TonicOff, Color::TonicOn, [57, 76]),
    ];
    let mut pos_to_off = HashMap::new();
    let mut pos_to_on = HashMap::new();
    let mut pos_to_other = HashMap::new();
    for (color, on_color, positions) in simulated {
        pos_to_other.insert(positions[0], positions[1]);
        pos_to_other.insert(positions[1], positions[0]);
        for position in positions {
            pos_to_off.insert(position, color);
            pos_to_on.insert(position, on_color);
            tx.send(Event::Light(LightEvent {
                mode: LightMode::On,
                position,
                color,
                label1: String::new(),
                label2: String::new(),
            }))?;
        }
    }
    drop(tx);
    while let Some(event) = events::receive_check_lag(&mut events_rx, None).await {
        let Event::Key(KeyEvent { key, velocity }) = event else {
            continue;
        };
        let color_map = if velocity == 0 {
            &pos_to_off
        } else {
            &pos_to_on
        };

        if let Some(color) = color_map.get(&key) {
            for position in [key, *pos_to_other.get(&key).unwrap()] {
                if let Some(tx) = events_tx.upgrade() {
                    tx.send(Event::Light(LightEvent {
                        mode: LightMode::On,
                        position,
                        color: *color,
                        label1: String::new(),
                        label2: String::new(),
                    }))?;
                }
            }
        }
    }
    Ok(())
}
