use midir::{MidiIO, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use midly::MidiMessage;
use midly::live::LiveEvent;
use std::error::Error;
use std::fmt::{Display, Formatter};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

mod message;

// See color.py for iterating on color choices.
pub mod colors {
    pub const LED_BLUE: u8 = 0x2d;
    pub const RGB_BLUE: &str = "#6161ff";
    pub const LED_GREEN: u8 = 0x15;
    pub const RGB_GREEN: &str = "#61ff61";
    pub const LED_PURPLE: u8 = 0x35;
    pub const RGB_PURPLE: &str = "#a161ff";
    pub const LED_PINK: u8 = 0x38;
    pub const RGB_PINK: &str = "#f98cff";
    pub const LED_RED: u8 = 0x06;
    pub const RGB_RED: &str = "#dd6161";
    pub const LED_ORANGE: u8 = 0x09;
    pub const RGB_ORANGE: &str = "#ffb361";
    pub const LED_CYAN: u8 = 0x25;
    pub const RGB_CYAN: &str = "#61eeff";
    pub const LED_YELLOW: u8 = 0x0d;
    pub const RGB_YELLOW: &str = "#ffff61";
    pub const LED_GRAY: u8 = 0x01;
    pub const RGB_GRAY: &str = "#b3b3b3";
    pub const LED_WHITE: u8 = 0x03;
    pub const RGB_WHITE: &str = "#ffffff";
}

#[derive(Copy, Clone, Debug)]
pub enum LightMode {
    On,
    Flashing,
    Pulsing,
}

#[derive(Clone, Debug)]
pub enum ToDevice {
    Shutdown,
    LightOn {
        mode: LightMode,
        position: u8,
        color: u8, // TODO: name
    },
    LightOff {
        position: u8,
    },
}
impl From<ToDevice> for Vec<u8> {
    fn from(value: ToDevice) -> Self {
        // See programmer docs. There are SysEx messages to control the LEDs, but in programmer
        // mode, you can send NoteOn events, which is what 0x90..0x92 are.
        match value {
            ToDevice::Shutdown => Vec::new(),
            ToDevice::LightOn {
                mode,
                position,
                color,
            } => {
                let mode: u8 = match mode {
                    LightMode::On => 0x90,
                    LightMode::Flashing => 0x91,
                    LightMode::Pulsing => 0x92,
                };
                vec![mode, position, color]
            }
            ToDevice::LightOff { position } => {
                vec![0x90, position, 0]
            }
        }
    }
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

    pub fn receiver(&self) -> broadcast::Receiver<FromDevice> {
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
                msg => self.output_connection.send(&Vec::from(msg))?,
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
                MidiMessage::NoteOn { key, vel } => {
                    let key = key.as_int();
                    let velocity = vel.as_int();
                    if velocity == 0 {
                        Some(FromDevice::KeyUp { key, velocity })
                    } else {
                        Some(FromDevice::KeyDown { key, velocity })
                    }
                },
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
        Ok(self.output_connection.send(message::ENTER_PROGRAMMER)?)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        let _ = self.output_connection.send(message::ENTER_LIVE);
    }
}
