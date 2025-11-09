use crate::controller::{Controller, Device};
use crate::events;
use crate::events::{Color, Event, KeyEvent, PressureEvent};
use midir::MidiOutputConnection;
use midly::MidiMessage;
use midly::live::LiveEvent;
use tokio::task::JoinHandle;

macro_rules! make_message {
    ( $( $bytes:literal ),* ) => {
        // All launchpad SysEx messages start and end the same way
        &[0xf0, 0x00, 0x20, 0x29, 0x02, 0x0e, $($bytes),*, 0xf7]
    };
}

const ENTER_LIVE: &[u8] = make_message!(0x0e, 0x00);
const ENTER_PROGRAMMER: &[u8] = make_message!(0x0e, 0x01);

#[derive(Default)]
pub struct Launchpad;

impl Launchpad {
    pub fn new() -> Self {
        Default::default()
    }

    fn set_light(
        output_connection: &mut MidiOutputConnection,
        position: u8,
        color: Color,
    ) -> anyhow::Result<()> {
        // The launchpad MK3 in programmer mode uses note events on channel 0 to turn lights
        // on, channel 1 for flashing, and channel 2 for pulsing. We only use channel 0 for
        // on/off events. See color.py for iterating on color choices.
        let code = 0x90; // note on, channel 0
        // See color.py for iterating on color choices.
        let color = color.launchpad_color();
        output_connection.send(&[code, position, color])?;
        Ok(())
    }

    fn clear_lights(output_connection: &mut MidiOutputConnection) -> anyhow::Result<()> {
        for position in 1..=108 {
            Self::set_light(output_connection, position, Color::Off)?;
        }
        Ok(())
    }

    pub async fn run(
        port_name: String,
        events_tx: events::WeakSender,
        mut events_rx: events::Receiver,
    ) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
        // Communicating with the MIDI device must be sync. The rest of the application must be
        // async. To bridge the gap, we create flume channels to relay back and forth.
        let (to_device_tx, to_device_rx) = flume::unbounded::<Event>();
        let (from_device_tx, from_device_rx) = flume::unbounded::<Event>();
        tokio::spawn(async move {
            while let Some(event) =
                events::receive_check_lag(&mut events_rx, Some("controller")).await
            {
                if let Err(e) = to_device_tx.send_async(event).await {
                    log::error!("failed to relay message to device: {e}");
                }
            }
        });
        tokio::spawn(async move {
            while let Ok(msg) = from_device_rx.recv_async().await {
                if let Some(tx) = events_tx.upgrade()
                    && let Err(e) = tx.send(msg)
                {
                    log::error!("failed to relay message from device: {e}");
                }
            }
        });
        Controller::<Self>::run(port_name, to_device_rx, from_device_tx)
    }
}

impl Device for Launchpad {
    type DeviceEvent = Event; // TODO -- make a separate event type

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

    fn handle_event(
        event: Event,
        output_connection: &mut MidiOutputConnection,
    ) -> anyhow::Result<()> {
        match event {
            Event::Light(e) => Self::set_light(output_connection, e.position, e.color),
            Event::ClearLights => Self::clear_lights(output_connection),
            _ => Ok(()),
        }
    }

    fn init(output_connection: &mut MidiOutputConnection) -> anyhow::Result<()> {
        Ok(output_connection.send(ENTER_PROGRAMMER)?)
    }

    fn shutdown(output_connection: &mut MidiOutputConnection) {
        let _ = output_connection.send(ENTER_LIVE);
    }
}
