use midir::{MidiIO, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use midly::MidiMessage;
use midly::live::LiveEvent;
use std::error::Error;

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

pub enum ToDevice {
    Shutdown,
    Data(Vec<u8>), // TODO: encapsulate the messages
}

pub struct Controller {
    input_connection: Option<MidiInputConnection<()>>,
    output_connection: MidiOutputConnection,
    tx: flume::Sender<ToDevice>,
    rx: flume::Receiver<ToDevice>,
    // TODO: subscribers
}

fn find_port<T: MidiIO>(ports: &T, name: &str) -> Result<T::Port, Box<dyn Error>> {
    ports
        .ports()
        .into_iter()
        .find(|p| {
            ports
                .port_name(p)
                .map(|n| n.contains(name))
                .unwrap_or(false)
        })
        .ok_or(format!("no port found containing '{name}'").into())
}

impl Controller {
    pub fn new(port_name: &str) -> Result<Self, Box<dyn Error>> {
        let midi_in = MidiInput::new("device-input")?;
        let in_port = find_port(&midi_in, port_name)?;
        log::debug!("opening input port: {}", midi_in.port_name(&in_port)?);
        // Handler keeps running until connection is dropped
        let input_connection = midi_in.connect(
            &in_port,
            "device-input",
            |stamp_ms, message, _| {
                Self::on_midi(stamp_ms, message);
            },
            (),
        )?;

        let midi_out = MidiOutput::new("device-output")?;
        let out_port = find_port(&midi_out, port_name)?;
        log::debug!("opening output port: {}", midi_out.port_name(&out_port)?);
        let output_connection = midi_out.connect(&out_port, "device-output")?;
        let (tx, rx) = flume::bounded(100);
        let mut controller = Self {
            input_connection: Some(input_connection),
            output_connection,
            tx,
            rx,
        };
        controller.init()?;
        Ok(controller)
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            match self.rx.recv()? {
                ToDevice::Shutdown => {
                    self.input_connection.take();
                    return Ok(());
                }
                ToDevice::Data(data) => {
                    // TODO: abstract
                    self.output_connection.send(&data)?
                }
            }
        }
    }

    fn on_midi(_stamp_ms: u64, event: &[u8]) {
        let Ok(event) = LiveEvent::parse(event) else {
            log::error!("invalid midi event received and ignored");
            return;
        };
        match event {
            // TODO: implement the ability to subscribe to messages
            LiveEvent::Midi { channel, message } => match message {
                MidiMessage::NoteOn { key, vel } => {
                    log::warn!("note on: note={key}, channel={channel}, velocity={vel}");
                }
                MidiMessage::NoteOff { key, vel } => {
                    log::warn!("note off: note={key}, channel={channel}, velocity={vel}");
                }
                MidiMessage::Aftertouch { key, vel } => {
                    // polyphonic after-touch; not supported on MK3 Pro as of 2025-07
                    log::warn!("after touch: note={key}, channel={channel}, velocity={vel}");
                }
                MidiMessage::Controller { controller, value } => {
                    log::warn!(
                        "controller: controller={controller}, channel={channel}, value={value}"
                    );
                }
                MidiMessage::ChannelAftertouch { vel } => {
                    log::warn!("channel after touch: value={vel}");
                }
                _ => {}
            },
            LiveEvent::Common(common) => {
                // Shouldn't happen in programmer mode
                log::debug!("common: {common:?}");
            }
            LiveEvent::Realtime(_) => {} // ignore
        }
    }

    pub fn sender(&self) -> flume::Sender<ToDevice> {
        self.tx.clone()
    }

    fn init(&mut self) -> Result<(), Box<dyn Error>> {
        let conn = &mut self.output_connection;
        conn.send(ENTER_PROGRAMMER)?;
        conn.send(LOWER_LEFT_SOLID_RED)?;
        conn.send(UPPER_RIGHT_FLASHING_GREEN)?;
        conn.send(MIDDLE_PULSING_BLUE)?;
        conn.send(XXX1)?;
        Ok(())
    }
}

impl Drop for Controller {
    fn drop(&mut self) {
        let _ = self.output_connection.send(ENTER_LIVE);
    }
}
