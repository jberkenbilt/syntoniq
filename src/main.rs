use clap::{Command, CommandFactory};
use clap::{Parser, Subcommand};
use clap_complete::{Generator, Shell, aot};
use log::LevelFilter;
use qlaunchpad::controller::colors::*;
use qlaunchpad::controller::{Controller, FromDevice, LightMode, ToDevice};
use qlaunchpad::midi_player;
use std::collections::HashMap;
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
        Commands::Colors => colors_main(&mut controller).await,
        Commands::Output => midi_player::play_midi(&mut controller).await,
    }?;
    controller.join().await
}

async fn events_main(controller: &mut Controller) -> Result<(), Box<dyn Error + Sync + Send>> {
    let mut rx = controller.receiver();
    while let Ok(event) = rx.recv().await {
        println!("{event}");
    }
    Ok(())
}

async fn colors_main(controller: &mut Controller) -> Result<(), Box<dyn Error + Sync + Send>> {
    let sender = controller.sender();
    for position in 1..=108 {
        sender.send(ToDevice::LightOff { position }).await.unwrap();
    }
    for [position, color] in [
        [11, LED_BLUE],
        [12, LED_PURPLE],
        [13, LED_RED],
        [14, LED_CYAN],
        [15, LED_GREEN],
        [16, LED_PINK],
        [17, LED_ORANGE],
        [18, LED_YELLOW],
    ] {
        sender
            .send(ToDevice::LightOn {
                mode: LightMode::On,
                position,
                color,
            })
            .await?;
    }
    let simulated = [
        (LED_CYAN, LED_YELLOW, [32, 51]),
        (LED_GRAY, LED_WHITE, [33, 52]),
        (LED_PURPLE, LED_PINK, [34, 53]),
        (LED_BLUE, LED_GREEN, [44, 63]),
        (LED_BLUE, LED_GREEN, [45, 64]),
        (LED_RED, LED_ORANGE, [46, 65]),
        (LED_GRAY, LED_WHITE, [47, 66]),
        (LED_CYAN, LED_YELLOW, [57, 76]),
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
            sender
                .send(ToDevice::LightOn {
                    mode: LightMode::On,
                    position,
                    color,
                })
                .await?;
        }
    }
    let mut rx = controller.receiver();
    while let Ok(event) = rx.recv().await {
        let (touched, color_map) = match event {
            FromDevice::Key { key, .. } => (key, &pos_to_on),
            _ => continue,
        };
        if let Some(color) = color_map.get(&touched) {
            for position in [touched, *pos_to_other.get(&touched).unwrap()] {
                sender
                    .send(ToDevice::LightOn {
                        mode: LightMode::On,
                        position,
                        color: *color,
                    })
                    .await?;
            }
        }
    }
    Ok(())
}
