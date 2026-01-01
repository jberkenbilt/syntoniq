use crate::controller::Device;
use crate::engine::Keyboard;
use crate::events;
#[cfg(test)]
use crate::events::TestEvent;
use crate::events::{
    ButtonData, Color, Event, FromDevice, KeyData, KeyEvent, LightData, LightEvent, Note,
    RawKeyEvent, RawLightEvent, RawPressureEvent, ToDevice,
};
use midir::MidiOutputConnection;
use midly::MidiMessage;
use midly::live::LiveEvent;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::sync::{Arc, LazyLock, RwLock};
use syntoniq_common::parsing::{Coordinate, Layout};

mod rgb_colors;

macro_rules! make_message {
    ( $( $bytes:literal ),* ) => {
        // All launchpad SysEx messages start and end the same way
        &[0xf0, 0x00, 0x20, 0x29, 0x02, 0x0e, $($bytes),*, 0xf7]
    };
}

const ENTER_LIVE: &[u8] = make_message!(0x0e, 0x00);
const ENTER_PROGRAMMER: &[u8] = make_message!(0x0e, 0x01);

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum CommandKeys {
    // Top Row, left to right
    Shift = 90,
    Transpose = 94, // Note
    Sustain = 95,   // Chord
    // Left column, top to bottom
    UpArrow = 80,
    DownArrow = 70,
    Clear = 60,
    Record = 10,
    // Right column, top to bottom
    LayoutScroll = 19,
    // Upper bottom controls
    LayoutMin = 101,
    LayoutMax = 108,
}

#[derive(Clone)]
pub struct Launchpad {
    events_tx: events::WeakSender,
    state: Arc<RwLock<State>>,
}
#[derive(Default, Clone)]
struct State {
    num_layouts: usize,
    cur_layout: Option<usize>,
    layout_offset: usize,
}
pub struct LaunchpadDevice;

impl Launchpad {
    pub fn new(events_tx: events::WeakSender) -> Self {
        let state: Arc<RwLock<State>> = Default::default();
        Launchpad {
            events_tx: events_tx.clone(),
            state: state.clone(),
        }
    }

    fn fix_layout_lights(&self) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let state = self.state.read().expect("lock").clone();
        let mut events = Vec::new();
        for i in 0..8 {
            let idx = i as usize + state.layout_offset;
            let key = u8::from(CommandKeys::LayoutMin) + i as u8;
            let event = if idx < state.num_layouts {
                let is_cur = state.cur_layout.map(|x| x == idx).unwrap_or(false);
                let color = if is_cur {
                    Color::ToggleOn
                } else {
                    Color::Active
                };
                RawLightEvent {
                    key,
                    color,
                    rgb_color: rgb_color(color),
                    label1: (idx + 1).to_string(),
                    label2: String::new(),
                }
            } else {
                RawLightEvent {
                    key,
                    color: Color::ControlOff,
                    rgb_color: rgb_color(Color::ControlOff),
                    label1: String::new(),
                    label2: String::new(),
                }
            };
            events.push(event);
        }
        if state.num_layouts > 8 {
            events.push(RawLightEvent {
                key: CommandKeys::LayoutScroll.into(),
                color: Color::Active,
                rgb_color: rgb_color(Color::Active),
                label1: "Scroll".to_string(),
                label2: "layouts".to_string(),
            });
        }
        if !events.is_empty() {
            tx.send(Event::ToDevice(ToDevice::Light(events)))?;
        }

        #[cfg(test)]
        events::send_test_event(&self.events_tx, TestEvent::LayoutsHandled);
        Ok(())
    }

    fn scroll_layouts(&self) -> anyhow::Result<()> {
        {
            let mut state = self.state.write().expect("poisoned lock");
            state.layout_offset += 8;
            if state.layout_offset >= state.num_layouts {
                state.layout_offset = 0;
            }
        }
        self.fix_layout_lights()?;
        Ok(())
    }

    fn handle_light_event(&self, event: LightEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let cmd = match event.light {
            LightData::Shift => CommandKeys::Shift,
            LightData::Sustain => CommandKeys::Sustain,
            LightData::Transpose => CommandKeys::Transpose,
        };
        tx.send(Event::ToDevice(ToDevice::Light(vec![RawLightEvent {
            key: cmd.into(),
            color: event.color,
            rgb_color: rgb_color(event.color),
            label1: event.label1,
            label2: event.label2,
        }])))?;
        Ok(())
    }

    fn is_note_key(position: u8) -> bool {
        (1..=8).contains(&(position / 10)) && (1..=8).contains(&(position % 10))
    }

    fn key_to_coordinate(key: u8) -> Coordinate {
        // Launchpad keys are RC where R is numbered from 1 (bottom) to 8 (top) and C is numbered
        // from 1 (left) to 8 (right). This turns out to match Syntoniq coordinates, not because
        // launchpad was first, but more likely because it's the most logical way to lay out a
        // musical keyboard.
        Coordinate {
            row: (key / 10) as i32,
            col: (key % 10) as i32,
        }
    }

    fn coordinate_to_key(position: Coordinate) -> u8 {
        // See key_to_coordinate. This won't overflow because it is only called internally when
        // we know we have values in range.
        (position.row * 10 + position.col) as u8
    }

    pub fn raw_key_to_button(position: u8) -> Option<ButtonData> {
        if Self::is_note_key(position) {
            Some(ButtonData::Note {
                position: Self::key_to_coordinate(position),
                orientation: None,
            })
        } else if CommandKeys::try_from(position).is_ok() {
            Some(ButtonData::Command { idx: position })
        } else {
            None
        }
    }

    pub fn button_to_raw_key(button: ButtonData) -> u8 {
        match button {
            ButtonData::Note { position, .. } => Self::coordinate_to_key(position),
            ButtonData::Command { idx } => idx,
        }
    }
}

impl LaunchpadDevice {
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
        let color = launchpad_color(color);
        output_connection.send(&[code, position, color])?;
        Ok(())
    }

    fn set_button_lights(
        output_connection: &mut MidiOutputConnection,
        events: &[RawLightEvent],
    ) -> anyhow::Result<()> {
        for e in events {
            Self::set_light(output_connection, e.key, e.color)?;
        }
        Ok(())
    }
}

impl Device for LaunchpadDevice {
    fn on_midi(&self, event: LiveEvent) -> Option<FromDevice> {
        match event {
            LiveEvent::Midi { message, .. } => match message {
                MidiMessage::NoteOn { key, vel } => {
                    let key = key.as_int();
                    let velocity = vel.as_int();
                    Some(FromDevice::Key(RawKeyEvent { key, velocity }))
                }
                MidiMessage::NoteOff { key, .. } => {
                    let key = key.as_int();
                    let velocity = 0;
                    Some(FromDevice::Key(RawKeyEvent { key, velocity }))
                }
                MidiMessage::Aftertouch { key, vel } => {
                    // polyphonic after-touch; not supported on MK3 Pro as of 2025-07
                    Some(FromDevice::Pressure(RawPressureEvent {
                        key: Some(key.as_int()),
                        velocity: vel.as_int(),
                    }))
                }
                MidiMessage::Controller { controller, value } => {
                    // Launchpad sends this in programmer mode for non-note keys.
                    let key = controller.as_int();
                    let velocity = value.as_int();
                    Some(FromDevice::Key(RawKeyEvent { key, velocity }))
                }
                MidiMessage::ChannelAftertouch { vel } => {
                    Some(FromDevice::Pressure(RawPressureEvent {
                        key: None,
                        velocity: vel.as_int(),
                    }))
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn handle_event(
        &self,
        event: ToDevice,
        output_connection: &mut MidiOutputConnection,
    ) -> anyhow::Result<()> {
        match event {
            ToDevice::Light(e) => LaunchpadDevice::set_button_lights(output_connection, &e),
        }
    }

    fn init(&self, output_connection: &mut MidiOutputConnection) -> anyhow::Result<()> {
        Ok(output_connection.send(ENTER_PROGRAMMER)?)
    }

    fn shutdown(&self, output_connection: &mut MidiOutputConnection) {
        let _ = output_connection.send(ENTER_LIVE);
    }
}

impl Keyboard for Launchpad {
    fn reset(&self) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        *self.state.write().expect("lock") = Default::default();
        let mut light_events = Vec::new();
        // Clear lights
        for key in 1..=108 {
            let color = if Self::is_note_key(key) {
                Color::Off
            } else {
                Color::ControlOff
            };
            light_events.push(RawLightEvent {
                key,
                color,
                rgb_color: rgb_color(color),
                label1: "".to_string(),
                label2: "".to_string(),
            })
        }
        // Draw the logo.
        for (color, positions) in [
            (
                Color::LogoGreen,
                vec![
                    82u8, 87, 71, 78, 61, 68, 51, 58, 41, 48, 32, 38, 23, 24, 27, 11, 12, 13, 14,
                    15, 16,
                ],
            ),
            (Color::LogoBlue, vec![74, 75, 63, 66, 53, 56, 44, 45]),
            (Color::LogoRed, vec![55, 46]),
            (Color::LogoPink, vec![64, 65, 54, 25, 26]),
        ] {
            for position in positions {
                light_events.push(RawLightEvent {
                    key: position,
                    color,
                    rgb_color: rgb_color(color),
                    label1: String::new(),
                    label2: String::new(),
                });
            }
        }
        for (cmd, label1, label2) in [
            (CommandKeys::UpArrow, "▲", ""),
            (CommandKeys::DownArrow, "▼", ""),
            (CommandKeys::Clear, "Reset", ""),
            (CommandKeys::Record, "Show", "Notes"),
        ] {
            light_events.push(RawLightEvent {
                key: cmd.into(),
                color: Color::Active,
                rgb_color: rgb_color(Color::Active),
                label1: label1.to_string(),
                label2: label2.to_string(),
            });
        }
        tx.send(Event::ToDevice(ToDevice::Light(light_events)))?;
        self.fix_layout_lights()?;
        Ok(())
    }

    fn multiple_keyboards(&self) -> bool {
        false
    }

    fn layout_supported(&self, layout: &Layout) -> bool {
        layout.keyboard == "launchpad"
    }

    fn note_positions(&self, _keyboard: &str) -> &'static [Coordinate] {
        static POSITIONS: LazyLock<Vec<Coordinate>> = LazyLock::new(|| {
            let mut v = Vec::with_capacity(64);
            for row in 1..=8 {
                for col in 1..=8 {
                    v.push(Coordinate { row, col });
                }
            }
            v
        });
        &POSITIONS
    }

    fn note_light_event(
        &self,
        note: Option<&Note>,
        position: Coordinate,
        velocity: u8,
    ) -> RawLightEvent {
        let key = Self::coordinate_to_key(position);
        match note {
            None => RawLightEvent {
                key,
                color: Color::Off,
                rgb_color: rgb_color(Color::Off),
                label1: String::new(),
                label2: String::new(),
            },
            Some(note) => {
                let color = if velocity == 0 {
                    note.off_color
                } else {
                    note.on_color
                };
                RawLightEvent {
                    key,
                    color,
                    rgb_color: rgb_color(color),
                    label1: note.placed.name.to_string(),
                    label2: note.placed.base_interval.to_string(),
                }
            }
        }
    }

    fn make_device(&self) -> Arc<dyn Device> {
        Arc::new(LaunchpadDevice)
    }

    fn handle_raw_event(&self, msg: FromDevice) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let FromDevice::Key(RawKeyEvent { key, velocity }) = msg else {
            return Ok(());
        };
        let off = velocity == 0;
        let send = |key: KeyData| -> anyhow::Result<()> {
            tx.send(Event::KeyEvent(KeyEvent { key, velocity }))?;
            Ok(())
        };
        if Self::is_note_key(key) {
            send(KeyData::Note {
                position: Self::key_to_coordinate(key),
            })?;
        } else if (u8::from(CommandKeys::LayoutMin)..=u8::from(CommandKeys::LayoutMax))
            .contains(&key)
        {
            if off {
                let state = self.state.read().expect("lock");
                let idx = (key - u8::from(CommandKeys::LayoutMin)) as usize + state.layout_offset;
                if idx < state.num_layouts {
                    send(KeyData::Layout { idx })?;
                }
            }
        } else if let Ok(command_key) = CommandKeys::try_from(key) {
            match command_key {
                CommandKeys::Shift => send(KeyData::Shift)?,
                CommandKeys::Transpose => send(KeyData::Transpose)?,
                CommandKeys::Sustain => send(KeyData::Sustain)?,
                CommandKeys::UpArrow => send(KeyData::OctaveShift { up: true })?,
                CommandKeys::DownArrow => send(KeyData::OctaveShift { up: false })?,
                CommandKeys::Clear => send(KeyData::Reset)?,
                CommandKeys::Record => send(KeyData::Print)?,
                CommandKeys::LayoutScroll => {
                    if off {
                        self.scroll_layouts()?
                    }
                }
                CommandKeys::LayoutMin | CommandKeys::LayoutMax => unreachable!(),
            }
        }
        Ok(())
    }

    fn main_event_loop(&self, event: Event) -> anyhow::Result<()> {
        match event {
            Event::Shutdown => return Ok(()),
            Event::SelectLayout(e) => {
                self.state.write().expect("lock").cur_layout = Some(e.idx);
                self.fix_layout_lights()?;
            }
            Event::ToDevice(_) | Event::KeyEvent(_) => {}
            Event::LightEvent(e) => self.handle_light_event(e)?,
            Event::SetLayoutNames(e) => {
                self.state.write().expect("lock").num_layouts = e.names.len();
                self.fix_layout_lights()?;
            }
            Event::Reset | Event::UpdateNote(_) | Event::PlayNote(_) => {}
            #[cfg(test)]
            Event::TestEngine(_) | Event::TestWeb(_) | Event::TestEvent(_) | Event::TestSync => {}
        }
        Ok(())
    }
}

mod colors {
    pub const WHITE: u8 = 0x03;
    pub const BLUE: u8 = 0x4f;
    pub const GREEN: u8 = 0x15;
    pub const PURPLE: u8 = 0x31;
    pub const PINK: u8 = 0x38;
    pub const RED: u8 = 0x06;
    pub const ORANGE: u8 = 0x09;
    pub const CYAN: u8 = 0x25;
    pub const YELLOW: u8 = 0x0d;
    pub const DULL_GRAY: u8 = 0x47;
    pub const HIGHLIGHT_GRAY: u8 = 0x01;
    pub const MAGENTA: u8 = 0x5e;
    pub const DIM_GREEN: u8 = 0x1b;
    pub const DIM_BLUE: u8 = 0x2b;
    pub const DIM_PINK: u8 = 0x37;
    pub const DIM_PURPLE: u8 = 0x33;
    pub const DIM_RED: u8 = 0x79;
    pub const DIM_ORANGE: u8 = 0x0b;
    pub const DIM_YELLOW: u8 = 0x0f;
    pub const DIM_CYAN: u8 = 0x27;
    pub const LOGO_PINK: u8 = 0x5d;
    pub const LOGO_BLUE: u8 = 0x29;
    pub const LOGO_GREEN: u8 = 0x1a;
}

pub fn launchpad_color(color: Color) -> u8 {
    match color {
        Color::Off => 0,
        Color::ControlOff => 0,
        Color::Active => colors::WHITE,
        Color::ToggleOff => colors::RED,
        Color::ToggleOn => colors::GREEN,
        Color::FourthOff => colors::DIM_GREEN,
        Color::FourthOn => colors::GREEN,
        Color::FifthOff => colors::DIM_BLUE,
        Color::FifthOn => colors::BLUE,
        Color::MajorThirdOff => colors::DIM_PINK,
        Color::MajorThirdOn => colors::PINK,
        Color::MinorSixthOff => colors::DIM_PURPLE,
        Color::MinorSixthOn => colors::PURPLE,
        Color::MinorThirdOff => colors::DIM_RED,
        Color::MinorThirdOn => colors::RED,
        Color::MajorSixthOff => colors::DIM_ORANGE,
        Color::MajorSixthOn => colors::ORANGE,
        Color::TonicOff => colors::DIM_YELLOW,
        Color::TonicOn => colors::YELLOW,
        Color::OtherOff => colors::DULL_GRAY,
        Color::OtherOn => colors::HIGHLIGHT_GRAY,
        Color::SingleStepOff => colors::DIM_CYAN,
        Color::SingleStepOn => colors::CYAN,
        Color::NoteSelected => colors::MAGENTA,
        Color::LogoPink => colors::LOGO_PINK,
        Color::LogoRed => colors::RED,
        Color::LogoGreen => colors::LOGO_GREEN,
        Color::LogoBlue => colors::LOGO_BLUE,
    }
}

pub fn rgb_color(color: Color) -> String {
    match color {
        Color::Off => events::OFF_RGB.to_string(),
        Color::ControlOff => "var(--control-background)".to_string(),
        _ => rgb_colors::RGB_COLORS[launchpad_color(color) as usize].to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine;
    use crate::events::LayoutNamesEvent;
    use crate::test_util::TestController;

    #[tokio::test]
    async fn test_scroll_layouts() -> anyhow::Result<()> {
        let mut tc = TestController::new().await;
        let events_tx = tc.tx().downgrade();
        let events_rx = tc.rx();
        let tx = events_tx.upgrade().unwrap();
        let launchpad = Arc::new(Launchpad::new(events_tx));
        engine::start_keyboard(None, launchpad.clone(), events_rx).await?;
        let layout_names: Vec<_> = (0..12).map(|x| x.to_string()).collect();
        tx.send(Event::SetLayoutNames(LayoutNamesEvent {
            names: layout_names.clone(),
        }))?;
        tc.wait_for_test_event(TestEvent::LayoutsHandled).await;

        launchpad.handle_raw_event(FromDevice::Key(RawKeyEvent {
            key: 105,
            velocity: 0,
        }))?;
        tc.wait_for_test_event(TestEvent::LayoutSelected).await;
        let ts = tc.get_engine_state().await;
        assert_eq!(ts.layout.unwrap(), 4);
        // Scroll layout
        launchpad.handle_raw_event(FromDevice::Key(RawKeyEvent {
            key: 19,
            velocity: 0,
        }))?;
        tc.wait_for_test_event(TestEvent::LayoutsHandled).await;
        assert_eq!(launchpad.state.read().expect("lock").layout_offset, 8);
        launchpad.handle_raw_event(FromDevice::Key(RawKeyEvent {
            key: 101,
            velocity: 0,
        }))?;
        tc.wait_for_test_event(TestEvent::LayoutSelected).await;
        let ts = tc.get_engine_state().await;
        assert_eq!(ts.layout.unwrap(), 8);

        launchpad.handle_raw_event(FromDevice::Key(RawKeyEvent {
            key: 19,
            velocity: 0,
        }))?;
        tc.wait_for_test_event(TestEvent::LayoutsHandled).await;
        assert_eq!(launchpad.state.read().expect("lock").layout_offset, 0);
        launchpad.handle_raw_event(FromDevice::Key(RawKeyEvent {
            key: 102,
            velocity: 0,
        }))?;
        tc.wait_for_test_event(TestEvent::LayoutSelected).await;
        let ts = tc.get_engine_state().await;
        assert_eq!(ts.layout.unwrap(), 1);

        tc.shutdown().await
    }
}
