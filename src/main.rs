use clap::{Arg, ArgAction, Command, value_parser};
use clap_complete::{Generator, Shell, aot};
use midir::{Ignore, MidiInput, MidiOutput};
use midly::MidiMessage;
use midly::live::LiveEvent;
use std::error::Error;
use std::io;
use std::io::Write;

// TODO: All messages start with the same six bytes and end with 0xf7.
const ENTER_LIVE: &[u8] = &[0xf0, 0x00, 0x20, 0x29, 0x02, 0x0e, 0x0e, 0x00, 0xf7];
const ENTER_PROGRAMMER: &[u8] = &[0xf0, 0x00, 0x20, 0x29, 0x02, 0x0e, 0x0e, 0x01, 0xf7];

// See programmer docs. These use note on messages to control LED
// color. There are also SysEx messages, but these work in programmer
// mode.
const LOWER_LEFT_SOLID_RED: &[u8] = &[
    0x90, // note on channel 1 -> static lighting
    11,   // note number 11 (lower left in programmer mode)
    0x05, // red
];
const UPPER_RIGHT_FLASHING_GREEN: &[u8] = &[
    0x91, // note on channel 2 -> flashing
    88, 0x13,
];
const MIDDLE_PULSING_BLUE: &[u8] = &[
    0x92, // note on channel 3 -> pulsing
    55, 0x2d,
];
// Also works for non-note buttons. These persist when you enter and exit programmer mode.
const XXX1: &[u8] = &[0x90, 95, 0x2d];
// Turn off: 0x90, note, 0x00

fn build_cli() -> Command {
    //TODO: decide if I want to use derive or arg macro, get completion working, figure out real
    // syntax
    Command::new("example")
        .arg(
            Arg::new("port")
                .long("port")
                .help("midi port name (amidiplay -l)"),
        )
        .arg(
            Arg::new("no-prog")
                .long("no-prog")
                .help("don't enter programmer mode")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("generator")
                .long("generate")
                .action(ArgAction::Set)
                .value_parser(value_parser!(Shell)),
        )
}

fn print_completions<G: Generator>(generator: G, cmd: &mut Command) {
    aot::generate(
        generator,
        cmd,
        cmd.get_name().to_string(),
        &mut io::stdout(),
    );
}

fn on_midi(_stamp_ms: u64, event: &[u8]) {
    let event = LiveEvent::parse(event).unwrap();
    match event {
        LiveEvent::Midi { channel, message } => match message {
            MidiMessage::NoteOn { key, vel } => {
                println!("note on: note={key}, channel={channel}, velocity={vel}");
            }
            MidiMessage::NoteOff { key, vel } => {
                println!("note off: note={key}, channel={channel}, velocity={vel}");
            }
            MidiMessage::Aftertouch { key, vel } => {
                // polyphonic after-touch; not supported on MK3 Pro as of 2025-07
                println!("after touch: note={key}, channel={channel}, velocity={vel}");
            }
            MidiMessage::Controller { controller, value } => {
                println!("controller: controller={controller}, channel={channel}, value={value}");
            }
            MidiMessage::ChannelAftertouch { vel } => {
                println!("channel after touch: value={vel}");
            }
            _ => {
                println!("XXX other 1: {message:?}");
            }
        },
        LiveEvent::Common(common) => {
            println!("common: {common:?}");
        }
        LiveEvent::Realtime(_) => {} // ignore
    }
}

fn run(port_name: &str, no_programmer_mode: bool) -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new("my-input")?;
    midi_in.ignore(Ignore::None);

    let in_ports = midi_in.ports();
    let in_port = in_ports
        .iter()
        .find(|p| {
            midi_in
                .port_name(p)
                .map(|n| n.contains(port_name))
                .unwrap_or(false)
        })
        .ok_or("No input port found matching the given name")?;

    println!("Opening input port: {}", midi_in.port_name(in_port)?);

    // Handler keeps running until connection is dropped
    let _conn_in = midi_in.connect(
        in_port,
        "midirdemo-in",
        |stamp_ms, message, _| {
            on_midi(stamp_ms, message);
        },
        (),
    )?;

    let midi_out = MidiOutput::new("my-output")?;

    let out_ports = midi_out.ports();
    let out_port = out_ports
        .iter()
        .find(|p| {
            midi_out
                .port_name(p)
                .map(|n| n.contains(port_name))
                .unwrap_or(false)
        })
        .ok_or("No output port found matching the given name")?;

    println!("Opening output port: {}", midi_out.port_name(out_port)?);

    let mut conn_out = midi_out.connect(out_port, "midirdemo-out")?;

    if !no_programmer_mode {
        conn_out.send(ENTER_PROGRAMMER)?;
    }
    conn_out.send(LOWER_LEFT_SOLID_RED)?;
    conn_out.send(UPPER_RIGHT_FLASHING_GREEN)?;
    conn_out.send(MIDDLE_PULSING_BLUE)?;
    conn_out.send(XXX1)?;

    println!("Sent Note On (middle C), listening for input events. Press ENTER to quit...");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // TODO: ensure this always runs on exit.
    conn_out.send(ENTER_LIVE)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    ctrlc::set_handler(move || {
        println!("TODO: restore live mode; hit enter to exit");
    })?;
    let matches = build_cli().get_matches();

    if let Some(generator) = matches.get_one::<Shell>("generator").copied() {
        let mut cmd = build_cli();
        eprintln!("Generating completion file for {generator}...");
        print_completions(generator, &mut cmd);
    }
    if let Some(port) = matches.get_one::<String>("port") {
        let no_prog = matches
            .get_one::<bool>("no-prog")
            .copied()
            .unwrap_or_default();
        run(port, no_prog)?;
    }
    Ok(())
}
