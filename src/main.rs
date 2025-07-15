use clap::{Command, CommandFactory};
use clap::{Parser, Subcommand};
use clap_complete::{Generator, Shell, aot};
use log::LevelFilter;
use qlaunchpad::controller::Controller;
use qlaunchpad::events::{Color, Event, Events, KeyEvent, LightEvent, LightMode};
use qlaunchpad::{events, midi_player};
use std::collections::HashMap;
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
    /// Display color choices
    Colors,
    /// Send notes to a virtual output port. Use QLaunchPad as the input device.
    Output,
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
        Commands::Output => midi_player::play_midi(events_rx.resubscribe()).await,
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
    for position in 1..=108 {
        tx.send(Event::Light(LightEvent {
            mode: LightMode::Off,
            position,
            color: Color::Off,
        }))
        .unwrap();
    }
    for (position, color) in [
        (11, Color::Blue),
        (12, Color::Purple),
        (13, Color::Red),
        (14, Color::Cyan),
        (15, Color::Green),
        (16, Color::Pink),
        (17, Color::Orange),
        (18, Color::Yellow),
    ] {
        tx.send(Event::Light(LightEvent {
            mode: LightMode::On,
            position,
            color,
        }))?;
    }
    let simulated = [
        (Color::Cyan, Color::Yellow, [32, 51]),
        (Color::Gray, Color::White, [33, 52]),
        (Color::Purple, Color::Pink, [34, 53]),
        (Color::Blue, Color::Green, [44, 63]),
        (Color::Blue, Color::Green, [45, 64]),
        (Color::Red, Color::Orange, [46, 65]),
        (Color::Gray, Color::White, [47, 66]),
        (Color::Cyan, Color::Yellow, [57, 76]),
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
            }))?;
        }
    }
    drop(tx);
    while let Some(event) = events::receive_ignore_lag(&mut events_rx).await {
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
                    }))?;
                }
            }
        }
    }
    Ok(())
}
