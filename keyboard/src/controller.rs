use crate::events;
use crate::events::{Color, Event, KeyEvent, LightEvent, PressureEvent, UpgradedSender};
use anyhow::bail;
use midir::{MidiIO, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use midly::MidiMessage;
use midly::live::LiveEvent;
use syntoniq_common::to_anyhow;
use tokio::task::JoinHandle;

mod message;

struct Device {
    input_connection: Option<MidiInputConnection<()>>,
    output_connection: MidiOutputConnection,
    to_device: flume::Receiver<Event>,
}

pub struct Controller;

pub(crate) fn find_port<T: MidiIO>(ports: &T, name: &str) -> anyhow::Result<T::Port> {
    let mut port_names = Vec::new();
    let result = ports.ports().into_iter().find(|p| {
        ports
            .port_name(p)
            .inspect(|n| {
                port_names.push(n.clone());
            })
            .map(|n| n.contains(name))
            .unwrap_or(false)
    });
    match result {
        None => {
            if port_names.is_empty() {
                eprintln!("no valid ports found");
            } else {
                eprintln!("Valid ports:");
                for p in port_names {
                    println!(" {p}");
                }
            }
            bail!("no port found containing '{name}'");
        }
        Some(r) => Ok(r),
    }
}

pub fn clear_lights(tx: &UpgradedSender) -> anyhow::Result<()> {
    for position in 1..=108 {
        tx.send(Event::Light(LightEvent {
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
        let (to_device_tx, to_device_rx) = flume::unbounded();
        let mut device = Device::new(&port_name, to_device_rx, events_tx).map_err(to_anyhow)?;
        tokio::spawn(async move {
            while let Some(event) =
                events::receive_check_lag(&mut events_rx, Some("controller")).await
            {
                if let Err(e) = to_device_tx.send_async(event).await {
                    log::error!("failed to relay message to device: {e}");
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
        to_device_rx: flume::Receiver<Event>,
        events_tx: events::WeakSender,
    ) -> anyhow::Result<Self> {
        let midi_in = MidiInput::new("Syntoniq Keyboard")?;
        let in_port = find_port(&midi_in, port_name)?;
        let full_port_name = midi_in.port_name(&in_port)?;
        log::debug!("opening input port: {full_port_name}",);
        // Handler keeps running until connection is dropped
        let input_connection = midi_in
            .connect(
                &in_port,
                &format!("{} to Syntoniq Keyboard", in_port.id()),
                move |stamp_ms, message, _| {
                    if let Some(event) = Self::on_midi(stamp_ms, message)
                        && let Some(tx) = events_tx.upgrade()
                        && let Err(e) = tx.send(event)
                    {
                        log::error!("error notifying of device event: {e}")
                    }
                },
                (),
            )
            .map_err(to_anyhow)?;

        let midi_out = MidiOutput::new("Syntoniq Keyboard")?;
        let out_port = find_port(&midi_out, port_name)?;
        let full_port_name = midi_out.port_name(&out_port)?;
        log::debug!("opening output port: {full_port_name}");
        let output_connection = midi_out
            .connect(
                &out_port,
                &format!("Syntoniq Keyboard to {}", out_port.id()),
            )
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
        while let Ok(e) = self.to_device.recv() {
            let Event::Light(event) = e else {
                continue;
            };
            // The launchpad MK3 in programmer mode uses note events on channel 0 to turn lights
            // on, channel 1 for flashing, and channel 2 for pulsing. We only use channel 0 for
            // on/off events. See color.py for iterating on color choices.
            let code = 0x90; // note on, channel 0
            // See color.py for iterating on color choices.
            let color = event.color.launchpad_color();
            self.output_connection
                .send(&[code, event.position, color])?;
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
                    Some(Event::Key(KeyEvent {
                        key,
                        velocity,
                        synthetic: false,
                    }))
                }
                MidiMessage::NoteOff { key, .. } => {
                    let key = key.as_int();
                    let velocity = 0;
                    Some(Event::Key(KeyEvent {
                        key,
                        velocity,
                        synthetic: false,
                    }))
                }
                MidiMessage::Aftertouch { key, vel } => {
                    // polyphonic after-touch; not supported on MK3 Pro as of 2025-07
                    Some(Event::Pressure(PressureEvent {
                        key: Some(key.as_int()),
                        velocity: vel.as_int(),
                    }))
                }
                MidiMessage::Controller { controller, value } => {
                    // Launchpad sends this in programmer mode for non-note keys.
                    let key = controller.as_int();
                    let velocity = value.as_int();
                    Some(Event::Key(KeyEvent {
                        key,
                        velocity,
                        synthetic: false,
                    }))
                }
                MidiMessage::ChannelAftertouch { vel } => Some(Event::Pressure(PressureEvent {
                    key: None,
                    velocity: vel.as_int(),
                })),
                _ => None,
            },
            _ => None,
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
