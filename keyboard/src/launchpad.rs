use crate::controller::{Controller, Device};
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
use tokio::task;
use tokio::task::JoinHandle;

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
        for i in 0..8 {
            let idx = i as usize + state.layout_offset;
            let button = ButtonData::Command {
                idx: u8::from(CommandKeys::LayoutMin) + i as u8,
            };
            let event = if idx < state.num_layouts {
                let is_cur = state.cur_layout.map(|x| x == idx).unwrap_or(false);
                let color = if is_cur {
                    Color::ToggleOn
                } else {
                    Color::Active
                };
                RawLightEvent {
                    button,
                    color,
                    rgb_color: rgb_color(color),
                    label1: (idx + 1).to_string(),
                    label2: String::new(),
                }
            } else {
                RawLightEvent {
                    button,
                    color: Color::Off,
                    rgb_color: rgb_color(Color::Off),
                    label1: String::new(),
                    label2: String::new(),
                }
            };
            tx.send(Event::ToDevice(ToDevice::Light(event)))?;
        }
        if state.num_layouts > 8 {
            tx.send(Event::ToDevice(ToDevice::Light(RawLightEvent {
                button: ButtonData::Command {
                    idx: CommandKeys::LayoutScroll.into(),
                },
                color: Color::Active,
                rgb_color: rgb_color(Color::Active),
                label1: "Scroll".to_string(),
                label2: "layouts".to_string(),
            })))?;
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
        tx.send(Event::ToDevice(ToDevice::Light(RawLightEvent {
            button: ButtonData::Command { idx: cmd.into() },
            color: event.color,
            rgb_color: rgb_color(event.color),
            label1: event.label1,
            label2: event.label2,
        })))?;
        Ok(())
    }

    pub async fn run(
        &self,
        controller: Option<Controller>,
        mut events_rx: events::Receiver,
    ) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
        let launchpad = self.clone();
        let controller_h = match controller {
            None => None,
            Some(c) => {
                // Start controller doesn't return until the device is initialized.
                Some(
                    launchpad
                        .clone()
                        .start_controller(c, events_rx.resubscribe())
                        .await?,
                )
            }
        };
        // Start the background task after the device is initialized so we're fully up before this
        // function returns.
        Ok(task::spawn(async move {
            while let Some(event) = events::receive_check_lag(&mut events_rx, Some("engine")).await
            {
                launchpad.main_event_loop(event)?;
            }
            if let Some(h) = controller_h {
                h.await??;
            }
            Ok(())
        }))
    }

    pub async fn start_controller(
        self,
        controller: Controller,
        mut events_rx: events::Receiver,
    ) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
        // Communicating with the MIDI device must be sync. The rest of the application must be
        // async. To bridge the gap, we create flume channels to relay back and forth.
        let (to_device_tx, to_device_rx) = flume::unbounded::<ToDevice>();
        let (from_device_tx, from_device_rx) = flume::unbounded::<FromDevice>();
        tokio::spawn(async move {
            while let Some(event) =
                events::receive_check_lag(&mut events_rx, Some("controller")).await
            {
                let Event::ToDevice(event) = event else {
                    continue;
                };
                if let Err(e) = to_device_tx.send_async(event).await {
                    log::error!("failed to relay message to device: {e}");
                }
            }
        });
        tokio::spawn(async move {
            while let Ok(msg) = from_device_rx.recv_async().await {
                if let Err(e) = self.handle_raw_event(msg) {
                    log::error!("error handling raw Launchpad event: {e}");
                }
            }
        });
        let device = Arc::new(LaunchpadDevice);
        controller.run(to_device_rx, from_device_tx, device)
    }

    pub fn main_event_loop(&self, event: Event) -> anyhow::Result<()> {
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

    fn is_note_key(position: u8) -> bool {
        (1..=8).contains(&(position / 10)) && (1..=8).contains(&(position % 10))
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
                CommandKeys::Clear => send(KeyData::Clear)?,
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
            })
        } else if CommandKeys::try_from(position).is_ok() {
            Some(ButtonData::Command { idx: position })
        } else {
            None
        }
    }

    pub fn button_to_raw_key(button: ButtonData) -> u8 {
        match button {
            ButtonData::Note { position } => Self::coordinate_to_key(position),
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

    fn set_button_light(
        output_connection: &mut MidiOutputConnection,
        button: ButtonData,
        color: Color,
    ) -> anyhow::Result<()> {
        let position = Launchpad::button_to_raw_key(button);
        Self::set_light(output_connection, position, color)
    }

    fn clear_lights(output_connection: &mut MidiOutputConnection) -> anyhow::Result<()> {
        for position in 1..=108 {
            Self::set_light(output_connection, position, Color::Off)?;
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
            ToDevice::Light(e) => {
                LaunchpadDevice::set_button_light(output_connection, e.button, e.color)
            }
            ToDevice::ClearLights => LaunchpadDevice::clear_lights(output_connection),
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
        // Draw the logo.
        tx.send(Event::ToDevice(ToDevice::ClearLights))?;
        for (color, positions) in [
            (
                Color::FifthOn, // green
                vec![63u8, 64, 65, 66, 52, 57, 42, 47, 32, 37, 23, 24, 25],
            ),
            (Color::FifthOff, vec![34, 35, 16, 17, 18]), // blue
            (Color::MajorThirdOn, vec![26]),             // pink
            (Color::MajorThirdOff, vec![72, 73, 83, 84, 85, 86, 76, 77]), // purple
            (Color::TonicOff, vec![74, 75]),             // cyan
        ] {
            for position in positions {
                let coord = Self::key_to_coordinate(position);
                tx.send(Event::ToDevice(ToDevice::Light(RawLightEvent {
                    button: ButtonData::Note { position: coord },
                    color,
                    rgb_color: rgb_color(color),
                    label1: String::new(),
                    label2: String::new(),
                })))?;
            }
        }
        for (cmd, label1, label2) in [
            (CommandKeys::UpArrow, "▲", ""),
            (CommandKeys::DownArrow, "▼", ""),
            (CommandKeys::Clear, "Reset", ""),
            (CommandKeys::Record, "Show", "Notes"),
        ] {
            tx.send(Event::ToDevice(ToDevice::Light(RawLightEvent {
                button: ButtonData::Command { idx: cmd.into() },
                color: Color::Active,
                rgb_color: rgb_color(Color::Active),
                label1: label1.to_string(),
                label2: label2.to_string(),
            })))?;
        }
        self.fix_layout_lights()?;
        Ok(())
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

    fn note_light_event(&self, note: Option<&Note>, position: Coordinate, velocity: u8) -> Event {
        match note {
            None => Event::ToDevice(ToDevice::Light(RawLightEvent {
                button: ButtonData::Note { position },
                color: Color::Off,
                rgb_color: rgb_color(Color::Off),
                label1: String::new(),
                label2: String::new(),
            })),
            Some(note) => {
                let color = if velocity == 0 {
                    note.off_color
                } else {
                    note.on_color
                };
                Event::ToDevice(ToDevice::Light(RawLightEvent {
                    button: ButtonData::Note { position },
                    color,
                    rgb_color: rgb_color(color),
                    label1: note.placed.name.to_string(),
                    label2: note.placed.base_interval.to_string(),
                }))
            }
        }
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
    pub const CYAN: u8 = 0x27;
    pub const YELLOW: u8 = 0x0d;
    pub const DULL_GRAY: u8 = 0x47;
    pub const HIGHLIGHT_GRAY: u8 = 0x01;
    pub const MAGENTA: u8 = 0x5e;
}

pub fn launchpad_color(color: Color) -> u8 {
    match color {
        Color::Off => 0,
        Color::Active => colors::WHITE,
        Color::ToggleOff => colors::RED,
        Color::ToggleOn => colors::GREEN,
        Color::FifthOff => colors::BLUE,
        Color::FifthOn => colors::GREEN,
        Color::MajorThirdOff => colors::PURPLE,
        Color::MajorThirdOn => colors::PINK,
        Color::MinorThirdOff => colors::RED,
        Color::MinorThirdOn => colors::ORANGE,
        Color::TonicOff => colors::CYAN,
        Color::TonicOn => colors::YELLOW,
        Color::OtherOff => colors::DULL_GRAY,
        Color::OtherOn => colors::WHITE,
        Color::SingleStepOff => colors::HIGHLIGHT_GRAY,
        Color::SingleStepOn => colors::WHITE,
        Color::NoteSelected => colors::MAGENTA,
    }
}

pub fn rgb_color(color: Color) -> &'static str {
    rgb_colors::RGB_COLORS[launchpad_color(color) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::LayoutNamesEvent;
    use crate::test_util::TestController;

    #[tokio::test]
    async fn test_scroll_layouts() -> anyhow::Result<()> {
        let mut tc = TestController::new().await;
        let events_tx = tc.tx().downgrade();
        let events_rx = tc.rx();
        let tx = events_tx.upgrade().unwrap();
        let launchpad = Launchpad::new(events_tx);
        launchpad.run(None, events_rx).await?;
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
