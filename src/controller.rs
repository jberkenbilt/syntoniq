use crate::events::{Color, Event, KeyEvent, LightEvent, LightMode, PressureEvent, UpgradedSender};
use crate::{events, to_anyhow};
use anyhow::anyhow;
use midir::{MidiIO, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use midly::MidiMessage;
use midly::live::LiveEvent;
use tokio::task::JoinHandle;

mod message;

struct Device {
    input_connection: Option<MidiInputConnection<()>>,
    output_connection: MidiOutputConnection,
    to_device: flume::Receiver<LightEvent>,
}

pub struct Controller;

fn find_port<T: MidiIO>(ports: &T, name: &str) -> anyhow::Result<T::Port> {
    ports
        .ports()
        .into_iter()
        .find(|p| {
            ports
                .port_name(p)
                .map(|n| n.contains(name))
                .unwrap_or(false)
        })
        .ok_or(anyhow!("no port found containing '{name}'"))
}

pub async fn clear_lights(tx: &UpgradedSender) -> anyhow::Result<()> {
    for position in 1..=108 {
        tx.send(Event::Light(LightEvent {
            mode: LightMode::Off,
            position,
            color: Color::Off,
            label1: String::new(),
            label2: String::new(),
        }))?;
    }
    Ok(())
}

impl Controller {
    pub async fn run(
        port_name: String,
        events_tx: events::WeakSender,
        mut events_rx: events::Receiver,
    ) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
        // Communicating with the MIDI device must be sync. The rest of the application must be
        // async. To bridge the gap, we create flume channels to relay back and forth.
        let (from_device_tx, from_device_rx) = flume::unbounded();
        let (to_device_tx, to_device_rx) = flume::unbounded();
        let mut device =
            Device::new(&port_name, to_device_rx, from_device_tx).map_err(to_anyhow)?;
        tokio::spawn(async move {
            while let Some(event) =
                events::receive_check_lag(&mut events_rx, Some("controller")).await
            {
                let Event::Light(event) = event else {
                    continue;
                };
                if let Err(e) = to_device_tx.send_async(event).await {
                    log::error!("failed to relay message to device: {e}");
                }
            }
        });
        tokio::spawn(async move {
            while let Ok(msg) = from_device_rx.recv_async().await {
                if let Some(tx) = events_tx.upgrade() {
                    if let Err(e) = tx.send(msg) {
                        log::error!("failed to relay message from device: {e}");
                    }
                }
            }
        });
        let handle: JoinHandle<anyhow::Result<()>> = tokio::task::spawn_blocking(move || {
            device.run()?;
            Ok(())
        });
        Ok(handle)
    }
}

impl Device {
    pub fn new(
        port_name: &str,
        to_device_rx: flume::Receiver<LightEvent>,
        from_device_tx: flume::Sender<Event>,
    ) -> anyhow::Result<Self> {
        let midi_in = MidiInput::new("q-launchpad")?;
        let in_port = find_port(&midi_in, port_name)?;
        log::debug!("opening input port: {}", midi_in.port_name(&in_port)?);
        // Handler keeps running until connection is dropped
        let input_connection = midi_in
            .connect(
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
            )
            .map_err(to_anyhow)?;

        let midi_out = MidiOutput::new("q-launchpad")?;
        let out_port = find_port(&midi_out, port_name)?;
        log::debug!("opening output port: {}", midi_out.port_name(&out_port)?);
        let output_connection = midi_out
            .connect(&out_port, "from-q-launchpad")
            .map_err(to_anyhow)?;
        let mut controller = Self {
            input_connection: Some(input_connection),
            output_connection,
            to_device: to_device_rx,
        };
        controller.init()?;
        Ok(controller)
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        while let Ok(event) = self.to_device.recv() {
            let mode = match event.mode {
                LightMode::Off | LightMode::On => 0x90,
                LightMode::Flashing => 0x91,
                LightMode::Pulsing => 0x92,
            };
            // See color.py for iterating on color choices.
            let color = event.color.launchpad_color();
            self.output_connection
                .send(&[mode, event.position, color])?;
        }
        log::debug!("device received shutdown request");
        // Dropping the input connection triggers the series events that leads
        // to clean shutdown: the on_midi loop closes, which closes the transmit
        // side of from_device, which causes all subscribers to exit.
        self.input_connection.take();
        Ok(())
    }

    fn on_midi(_stamp_ms: u64, event: &[u8]) -> Option<Event> {
        let Ok(event) = LiveEvent::parse(event) else {
            log::error!("invalid midi event received and ignored");
            return None;
        };
        match event {
            LiveEvent::Midi { message, .. } => match message {
                MidiMessage::NoteOn { key, vel } => {
                    let key = key.as_int();
                    let velocity = vel.as_int();
                    Some(Event::Key(KeyEvent { key, velocity }))
                }
                MidiMessage::Aftertouch { key, vel } => {
                    // polyphonic after-touch; not supported on MK3 Pro as of 2025-07
                    Some(Event::Pressure(PressureEvent {
                        key: Some(key.as_int()),
                        velocity: vel.as_int(),
                    }))
                }
                MidiMessage::Controller { controller, value } => {
                    let key = controller.as_int();
                    let velocity = value.as_int();
                    Some(Event::Key(KeyEvent { key, velocity }))
                }
                MidiMessage::ChannelAftertouch { vel } => Some(Event::Pressure(PressureEvent {
                    key: None,
                    velocity: vel.as_int(),
                })),
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

    fn init(&mut self) -> anyhow::Result<()> {
        Ok(self.output_connection.send(message::ENTER_PROGRAMMER)?)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        let _ = self.output_connection.send(message::ENTER_LIVE);
    }
}
