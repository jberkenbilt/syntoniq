use midir::{MidiIO, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use midly::MidiMessage;
use midly::live::LiveEvent;
use std::error::Error;
use std::fmt::{Display, Formatter};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

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

#[derive(Clone, Debug)]
pub enum ToDevice {
    Shutdown,
    Data(Vec<u8>), // TODO: encapsulate the messages
}

#[derive(Clone, Debug)]
pub enum FromDevice {
    KeyDown { key: u8, velocity: u8 },
    KeyUp { key: u8, velocity: u8 },
    Pressure { key: Option<u8>, velocity: u8 },
}

impl Display for FromDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FromDevice::KeyDown { key, velocity } => {
                write!(f, "key down: key={key:02}, velocity={velocity}")
            }
            FromDevice::KeyUp { key, velocity } => {
                write!(f, "key up: key={key:02}, velocity={velocity}")
            }
            FromDevice::Pressure { key, velocity } => {
                write!(
                    f,
                    "pressure: key={}, velocity={velocity}",
                    key.map(|x| format!("{x:02}"))
                        .unwrap_or("global".to_string())
                )
            }
        }
    }
}

struct Device {
    input_connection: Option<MidiInputConnection<()>>,
    output_connection: MidiOutputConnection,
    to_device: flume::Receiver<ToDevice>,
}

pub struct Controller {
    handle: JoinHandle<Result<(), Box<dyn Error + Sync + Send>>>,
    from_device: broadcast::Receiver<FromDevice>,
    to_device: mpsc::Sender<ToDevice>,
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

fn to_sync_send(e: Box<dyn Error>) -> Box<dyn Error + Sync + Send> {
    e.to_string().into()
}

impl Controller {
    pub async fn new(port_name: String) -> Result<Self, Box<dyn Error + Sync + Send>> {
        // Communicating with the MIDI device must be sync. The rest of the application must be
        // async. To bridge the gap, we create flume channels to relay back and forth.
        let (from_device_tx, from_device_rx) = broadcast::channel::<FromDevice>(5);
        let (to_device_tx, mut to_device_rx) = mpsc::channel(100);
        let (from_device_sync_tx, from_device_sync_rx) = flume::unbounded::<FromDevice>();
        let (to_device_sync_tx, to_device_sync_rx) = flume::unbounded::<ToDevice>();
        tokio::spawn(async move {
            while let Some(msg) = to_device_rx.recv().await {
                if let Err(e) = to_device_sync_tx.send_async(msg).await {
                    log::error!("failed to relay message to device: {e}");
                }
            }
        });
        tokio::spawn(async move {
            while let Ok(msg) = from_device_sync_rx.recv_async().await {
                if let Err(e) = from_device_tx.send(msg) {
                    log::error!("failed to relay message from device: {e}");
                }
            }
        });
        let handle: JoinHandle<Result<(), Box<dyn Error + Sync + Send>>> =
            tokio::task::spawn_blocking(move || {
                let mut device = Device::new(&port_name, to_device_sync_rx, from_device_sync_tx)
                    .map_err(to_sync_send)?;
                device.run().map_err(to_sync_send)?;
                Ok(())
            });
        Ok(Self {
            handle,
            from_device: from_device_rx,
            to_device: to_device_tx,
        })
    }

    pub fn sender(&self) -> mpsc::Sender<ToDevice> {
        self.to_device.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<FromDevice> {
        self.from_device.resubscribe()
    }

    pub async fn join(self) -> Result<(), Box<dyn Error + Sync + Send>> {
        self.handle.await?
    }
}

impl Device {
    pub fn new(
        port_name: &str,
        to_device_rx: flume::Receiver<ToDevice>,
        from_device_tx: flume::Sender<FromDevice>,
    ) -> Result<Self, Box<dyn Error>> {
        let midi_in = MidiInput::new("device-input")?;
        let in_port = find_port(&midi_in, port_name)?;
        log::debug!("opening input port: {}", midi_in.port_name(&in_port)?);
        // Handler keeps running until connection is dropped
        let input_connection = midi_in.connect(
            &in_port,
            "device-input",
            move |stamp_ms, message, _| {
                if let Some(event) = Self::on_midi(stamp_ms, message) {
                    if let Err(e) = from_device_tx.send(event) {
                        log::error!("error notifying of device event: {e}")
                    }
                }
            },
            (),
        )?;

        let midi_out = MidiOutput::new("device-output")?;
        let out_port = find_port(&midi_out, port_name)?;
        log::debug!("opening output port: {}", midi_out.port_name(&out_port)?);
        let output_connection = midi_out.connect(&out_port, "device-output")?;
        let mut controller = Self {
            input_connection: Some(input_connection),
            output_connection,
            to_device: to_device_rx,
        };
        controller.init()?;
        Ok(controller)
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            match self.to_device.recv()? {
                ToDevice::Shutdown => {
                    log::debug!("device received shutdown request");
                    // Dropping the input connection triggers the series events that leads
                    // to clean shutdown: the on_midi loop closes, which closes the transmit
                    // side of from_device, which causes all subscribers to exit.
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

    fn on_midi(_stamp_ms: u64, event: &[u8]) -> Option<FromDevice> {
        let Ok(event) = LiveEvent::parse(event) else {
            log::error!("invalid midi event received and ignored");
            return None;
        };
        match event {
            LiveEvent::Midi { message, .. } => match message {
                MidiMessage::NoteOn { key, vel } => Some(FromDevice::KeyDown {
                    key: key.as_int(),
                    velocity: vel.as_int(),
                }),
                MidiMessage::NoteOff { key, vel } => Some(FromDevice::KeyUp {
                    key: key.as_int(),
                    velocity: vel.as_int(),
                }),
                MidiMessage::Aftertouch { key, vel } => {
                    // polyphonic after-touch; not supported on MK3 Pro as of 2025-07
                    Some(FromDevice::Pressure {
                        key: Some(key.as_int()),
                        velocity: vel.as_int(),
                    })
                }
                MidiMessage::Controller { controller, value } => {
                    let key = controller.as_int();
                    let velocity = value.as_int();
                    if value == 0 {
                        Some(FromDevice::KeyUp { key, velocity })
                    } else {
                        Some(FromDevice::KeyDown { key, velocity })
                    }
                }
                MidiMessage::ChannelAftertouch { vel } => Some(FromDevice::Pressure {
                    key: None,
                    velocity: vel.as_int(),
                }),
                _ => None,
            },
            LiveEvent::Common(common) => {
                // Shouldn't happen in programmer mode
                log::warn!("unhandled device event: common: {common:?}");
                None
            }
            LiveEvent::Realtime(_) => None,
        }
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

impl Drop for Device {
    fn drop(&mut self) {
        let _ = self.output_connection.send(ENTER_LIVE);
    }
}
