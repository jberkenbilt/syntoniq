use crate::controller::{Controller, Device};
use crate::events;
use crate::events::{Color, Event, KeyEvent, LightEvent, PressureEvent};
use midir::MidiOutputConnection;
use midly::MidiMessage;
use midly::live::LiveEvent;
use std::collections::HashMap;
use tokio::task;
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
        events_rx: events::Receiver,
    ) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
        let controller_h =
            Self::start_controller(port_name, events_tx.clone(), events_rx.resubscribe()).await?;
        Ok(task::spawn(async move {
            Self::main_event_loop(events_tx, events_rx).await?;
            controller_h.await?
        }))
    }

    pub async fn start_controller(
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

    async fn colors_main(
        events_tx: events::WeakSender,
        mut events_rx: events::Receiver,
    ) -> anyhow::Result<()> {
        let Some(tx) = events_tx.upgrade() else {
            return Ok(());
        };
        tx.send(Event::ClearLights)?;
        // Light all control keys
        for range in [1..=8, 101..=108, 90..=99] {
            for position in range {
                tx.send(Event::Light(LightEvent {
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
                    position,
                    color,
                    label1: String::new(),
                    label2: String::new(),
                }))?;
            }
        }
        drop(tx);
        while let Some(event) = events::receive_check_lag(&mut events_rx, None).await {
            let Event::Key(KeyEvent { key, velocity, .. }) = event else {
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

    pub async fn main_event_loop(
        events_tx: events::WeakSender,
        mut events_rx: events::Receiver,
    ) -> anyhow::Result<()> {
        while let Some(event) = events::receive_check_lag(&mut events_rx, Some("engine")).await {
            match event {
                Event::Shutdown => return Ok(()),
                Event::ColorsMain => {
                    return Self::colors_main(events_tx.clone(), events_rx.resubscribe()).await;
                }
                Event::Light(_) => {}
                Event::Key(_) => {}
                Event::Pressure(_) => {}
                Event::Reset => {}
                Event::ClearLights => {}
                Event::SetLayoutNames(_) => {}
                Event::SelectLayout(_) => {}
                Event::ScrollLayouts => {}
                Event::UpdateNote(_) => {}
                Event::PlayNote(_) => {}
                #[cfg(test)]
                Event::TestEngine(_) => {}
                #[cfg(test)]
                Event::TestWeb(_) => {}
                #[cfg(test)]
                Event::TestEvent(_) => {}
                #[cfg(test)]
                Event::TestSync => {}
            }
            // TODO
        }
        Ok(())
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
