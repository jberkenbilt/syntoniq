use crate::controller::Device;
use crate::engine::Keyboard;
use crate::events;
#[cfg(test)]
use crate::events::TestEvent;
use crate::events::{
    ButtonData, Color, Event, FromDevice, KeyData, KeyEvent, LightData, LightEvent, Note,
    RawKeyEvent, RawLightEvent, ToDevice,
};
use crate::launchpad::launchpad_color;
use bimap::BiMap;
use midir::MidiOutputConnection;
use midly::MidiMessage;
use midly::live::LiveEvent;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::cmp;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};
use syntoniq_common::parsing::{Coordinate, Layout};

macro_rules! make_message {
     ( $( $bytes:literal ),* ) => {
         // Replace 0x7D with the three-byte manufacturer ID of HexBoard when obtained.
         &[0xf0, 0x7D, $($bytes),*, 0xf7]
     };
 }

const ENTER_DELEGATED: &[u8] = make_message!(0x01);
const EXIT_DELEGATED: &[u8] = make_message!(0x02);

static KEY_MAP: LazyLock<HashMap<Orientation, BiMap<u8, Coordinate>>> = LazyLock::new(init_key_map);

#[derive(Debug, Copy, Clone)]
/// HSV: range for each is 0..=127
pub struct HSV {
    pub hue: u8,
    pub sat: u8,
    pub val: u8,
}
impl HSV {
    fn to_rgb(self) -> String {
        // This function was AI-generated.
        let h = self.hue;
        let s = self.sat;
        let v = self.val;

        // 1. Handle grayscale (Saturation = 0)
        if s == 0 {
            let v_out = (v as u16 * 255 / 127) as u8;
            return format!("#{0:02x}{0:02x}{0:02x}", v_out);
        }

        // 2. Determine Sector (0-5)
        // 128 / 6 = 21.33. We use (h * 6) >> 7 to safely map 0-127 into 0-5.
        let region = (h as u16 * 6) >> 7;

        // 3. Calculate "fractional" part within the sector (0-127 range)
        // Equivalent to (h mod 21.33) scaled up
        let rem = ((h as u16 * 6) & 127) as u8;

        // 4. Calculate p, q, t vars (scaled 0-127)
        // p = v * (1 - s)
        let p = (v as u16 * (127 - s as u16) / 127) as u8;
        // q = v * (1 - s * f)
        let q = (v as u16 * (127 - (s as u16 * rem as u16) / 127) / 127) as u8;
        // t = v * (1 - s * (1 - f))
        let t = (v as u16 * (127 - (s as u16 * (127 - rem as u16)) / 127) / 127) as u8;

        // 5. Assign to R, G, B based on sector
        let (r, g, b) = match region {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };

        // 6. Scale to 0-255 for standard Hex output
        let scale = |val: u8| (val as u16 * 255 / 127) as u8;
        format!("#{:02x}{:02x}{:02x}", scale(r), scale(g), scale(b))
    }
}

#[derive(Clone)]
pub struct HexBoard {
    events_tx: events::WeakSender,
    orientation: Arc<RwLock<Orientation>>,
    state: Arc<RwLock<State>>,
}
#[derive(Default, Clone)]
struct State {
    layout_names: Vec<String>,
    cur_layout: Option<usize>,
    layout_mode: bool,
}
#[derive(Default, Copy, Clone, PartialOrd, PartialEq, Eq, Hash)]
pub enum Orientation {
    #[default]
    Horiz,
    R60,
}
pub struct HexBoardDevice {
    orientation: Arc<RwLock<Orientation>>,
}

pub struct LedMessage {
    key: u8,
    color: Color,
}

// The HexBoard has 7 command keys. These are our assignments from top to bottom.
#[repr(u8)]
#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
pub enum CommandKey {
    Reset,
    Layout,
    Sustain,
    OctaveUp,
    OctaveDown,
    Shift,
    Transpose,
}

impl HexBoard {
    pub fn new(events_tx: events::WeakSender) -> Self {
        let state: Arc<RwLock<State>> = Default::default();
        HexBoard {
            events_tx: events_tx.clone(),
            orientation: Default::default(),
            state: state.clone(),
        }
    }

    fn key_to_layout_idx(num_layouts: usize, key: u8) -> Option<usize> {
        // Key is layout number, but skip over command keys, which are multiples of 20.
        if key.is_multiple_of(20) {
            None
        } else {
            let idx = (key - (key / 20) - 1) as usize;
            if idx < num_layouts { Some(idx) } else { None }
        }
    }

    fn enter_layout_mode(&self) -> anyhow::Result<()> {
        println!("Available layouts:");
        let num_layouts = {
            let mut state = self.state.write().unwrap();
            state.layout_mode = true;
            for n in &state.layout_names {
                println!("  {n}");
            }
            state.layout_names.len()
        };
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        // Set orientation to Horizontal for layout selection. Layout selection always ends with
        // a layout key, even if canceled, so the layout will be properly restored.
        *self.orientation.write().unwrap() = Orientation::Horiz;
        let raw_events: Vec<_> = (0u8..=139)
            .map(|key| {
                let color = if Self::key_to_layout_idx(num_layouts, key).is_some() {
                    Color::Active
                } else {
                    Color::Off
                };
                let position = Self::key_to_coordinate(key, Orientation::Horiz);
                RawLightEvent {
                    button: ButtonData::Note { position },
                    color,
                    rgb_color: "".to_string(),
                    label1: "".to_string(),
                    label2: "".to_string(),
                }
            })
            .collect();
        tx.send(Event::ToDevice(ToDevice::Light(raw_events)))?;
        #[cfg(test)]
        events::send_test_event(&self.events_tx, TestEvent::LayoutsHandled);
        Ok(())
    }

    fn handle_layout_key(&self, key: u8) -> Option<usize> {
        let mut result = self.state.read().unwrap().cur_layout;
        let exit_layout = if key.is_multiple_of(20)
            && matches!(CommandKey::try_from(key / 20), Ok(CommandKey::Layout))
        {
            true
        } else if Self::is_note_key(key) {
            result = Self::key_to_layout_idx(self.state.read().unwrap().layout_names.len(), key);
            result.is_some()
        } else {
            false
        };
        if exit_layout {
            self.state.write().unwrap().layout_mode = false;
        }
        #[cfg(test)]
        events::send_test_event(&self.events_tx, TestEvent::LayoutsHandled);
        result
    }

    fn handle_light_event(&self, event: LightEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let cmd = match event.light {
            LightData::Shift => CommandKey::Shift,
            LightData::Sustain => CommandKey::Sustain,
            LightData::Transpose => CommandKey::Transpose,
        };
        tx.send(Event::ToDevice(ToDevice::Light(vec![RawLightEvent {
            button: ButtonData::Command { idx: cmd.into() },
            color: event.color,
            rgb_color: launchpad_color(event.color).to_string(),
            label1: event.label1,
            label2: event.label2,
        }])))?;
        Ok(())
    }

    fn is_note_key(position: u8) -> bool {
        !position.is_multiple_of(20)
    }

    fn key_to_coordinate(key: u8, orientation: Orientation) -> Coordinate {
        KEY_MAP
            .get(&orientation)
            .unwrap()
            .get_by_left(&key)
            .copied()
            .unwrap_or(Coordinate { row: 0, col: 0 })
    }

    fn coordinate_to_key(position: Coordinate, orientation: Orientation) -> u8 {
        KEY_MAP
            .get(&orientation)
            .unwrap()
            .get_by_right(&position)
            .copied()
            .unwrap_or_default()
    }

    pub fn button_to_raw_key(button: ButtonData, orientation: Orientation) -> u8 {
        match button {
            ButtonData::Note { position } => Self::coordinate_to_key(position, orientation),
            ButtonData::Command { idx } => {
                // HexBoard command keys are 0, 20, ... 120
                idx * 20
            }
        }
    }
}

impl HexBoardDevice {
    fn set_lights(
        output_connection: &mut MidiOutputConnection,
        messages: &[LedMessage],
    ) -> anyhow::Result<()> {
        // The longest SysEx we can send is 60 bytes. We have 5 bytes per light + header + end.
        // At initial implementation, header is 3 bytes, but it would be 5 if HexBoard gets
        // a manufacturer ID. That leaves us enough bytes for 10 lights. This works well because
        // there are 10 notes per row, including command keys, whose key numbers are the 0th column
        // of the 9-note rows.

        let mut raw_messages = Vec::new();
        for start in (0..messages.len()).step_by(10) {
            // SysEx 03 key-MSB key-LSB H S V ...
            let mut v = vec![0xF0, 0x7D, 0x03];
            for m in &messages[start..cmp::min(messages.len(), start + 10)] {
                let msb = m.key / 128;
                let lsb = m.key % 128;
                let hsv = hexboard_color(m.color);
                v.extend_from_slice(&[msb, lsb, hsv.hue, hsv.sat, hsv.val]);
            }
            v.push(0xF7);
            raw_messages.push(v);
        }
        for message in raw_messages {
            output_connection.send(message.as_slice())?;
        }
        Ok(())
    }

    fn set_button_lights(
        output_connection: &mut MidiOutputConnection,
        events: &[RawLightEvent],
        orientation: Orientation,
    ) -> anyhow::Result<()> {
        let mut led_messages: Vec<_> = events
            .iter()
            .map(|e| LedMessage {
                key: HexBoard::button_to_raw_key(e.button, orientation),
                color: e.color,
            })
            .collect();
        led_messages.sort_by_key(|x| x.key);
        Self::set_lights(output_connection, &led_messages)
    }

    fn clear_lights(output_connection: &mut MidiOutputConnection) -> anyhow::Result<()> {
        static ALL_OFF: LazyLock<Vec<LedMessage>> = LazyLock::new(|| {
            (0u8..=139)
                .map(|key| LedMessage {
                    key,
                    color: Color::Off,
                })
                .collect()
        });
        Self::set_lights(output_connection, &ALL_OFF)
    }
}

impl Device for HexBoardDevice {
    fn on_midi(&self, event: LiveEvent) -> Option<FromDevice> {
        match event {
            LiveEvent::Midi { message, channel } => match message {
                MidiMessage::NoteOn { key, vel } => {
                    let key = key.as_int() + channel.as_int() * 100;
                    let velocity = vel.as_int();
                    Some(FromDevice::Key(RawKeyEvent { key, velocity }))
                }
                MidiMessage::NoteOff { key, .. } => {
                    let key = key.as_int() + channel.as_int() * 100;
                    let velocity = 0;
                    Some(FromDevice::Key(RawKeyEvent { key, velocity }))
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
            ToDevice::Light(e) => HexBoardDevice::set_button_lights(
                output_connection,
                &e,
                *self.orientation.read().expect("lock"),
            ),
            ToDevice::ClearLights => HexBoardDevice::clear_lights(output_connection),
        }
    }

    fn init(&self, output_connection: &mut MidiOutputConnection) -> anyhow::Result<()> {
        Ok(output_connection.send(ENTER_DELEGATED)?)
    }

    fn shutdown(&self, output_connection: &mut MidiOutputConnection) {
        let _ = output_connection.send(EXIT_DELEGATED);
    }
}

impl Keyboard for HexBoard {
    fn reset(&self) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        *self.state.write().expect("lock") = Default::default();
        *self.orientation.write().expect("lock") = Orientation::Horiz;
        // Draw the logo.
        tx.send(Event::ToDevice(ToDevice::ClearLights))?;
        let mut light_events = Vec::new();
        for (color, keys) in [
            (
                Color::FifthOn, // green
                vec![
                    3u8, 4, 5, 6, 7, 17, 28, 38, 49, 59, 69, 78, 88, 97, 107, 116, 126, 125, 124,
                    123, 122, 113, 104, 94, 85, 84, 83, 72, 62, 51, 41, 31, 22, 12,
                ],
            ),
            (
                Color::FifthOff, //blue
                vec![24, 25, 26, 36, 47, 56, 66, 65, 64, 53, 43, 33],
            ),
            //(Color::MajorThirdOn, vec![1, 2, 8, 9, 10, 11, 18, 19, 21b, 29, 30, 39]),     // pink
            (
                Color::LogoBackground, // light pink
                vec![
                    13, 14, 15, 16, 23, 27, 32, 34, 35, 37, 42, 44, 45, 46, 48, 52, 54, 57, 58, 63,
                    67, 68, 73, 74, 75, 77, 86, 87, 95, 96, 105, 106, 114, 115,
                ],
            ),
            (Color::MinorThirdOff, vec![55, 76]), // red
        ] {
            for key in keys {
                let position = Self::key_to_coordinate(key, Orientation::Horiz);
                light_events.push(RawLightEvent {
                    button: ButtonData::Note { position },
                    color,
                    rgb_color: hexboard_color(color).to_rgb(),
                    label1: String::new(),
                    label2: String::new(),
                });
            }
        }
        for (cmd, label1, label2) in [
            (CommandKey::Layout, "Layout", ""),
            (CommandKey::OctaveUp, "▲", ""),
            (CommandKey::OctaveDown, "▼", ""),
            (CommandKey::Reset, "Reset", ""),
        ] {
            light_events.push(RawLightEvent {
                button: ButtonData::Command { idx: cmd.into() },
                color: Color::Active,
                rgb_color: hexboard_color(Color::Active).to_rgb(),
                label1: label1.to_string(),
                label2: label2.to_string(),
            });
        }
        tx.send(Event::ToDevice(ToDevice::Light(light_events)))?;
        println!("HexBoard command keys, top to bottom:");
        for i in 0u8..7 {
            println!("  {:?}", CommandKey::try_from(i).unwrap())
        }
        Ok(())
    }

    fn layout_supported(&self, layout: &Layout) -> bool {
        layout.keyboard.starts_with("hexboard")
    }

    fn note_positions(&self, keyboard: &str) -> &'static [Coordinate] {
        let orientation = if keyboard == "hexboard" {
            Orientation::Horiz
        } else {
            Orientation::R60
        };
        *self.orientation.write().unwrap() = orientation;
        static POSITIONS: LazyLock<HashMap<Orientation, Vec<Coordinate>>> = LazyLock::new(|| {
            KEY_MAP
                .iter()
                .map(|(orientation, keys)| (*orientation, keys.right_values().copied().collect()))
                .collect()
        });
        POSITIONS.get(&orientation).unwrap().as_slice()
    }

    fn note_light_event(
        &self,
        note: Option<&Note>,
        position: Coordinate,
        velocity: u8,
    ) -> RawLightEvent {
        match note {
            None => RawLightEvent {
                button: ButtonData::Note { position },
                color: Color::Off,
                rgb_color: events::OFF_RGB.to_string(),
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
                    button: ButtonData::Note { position },
                    color,
                    rgb_color: hexboard_color(color).to_rgb(),
                    label1: note.placed.name.to_string(),
                    label2: note.placed.base_interval.to_string(),
                }
            }
        }
    }

    fn make_device(&self) -> Arc<dyn Device> {
        Arc::new(HexBoardDevice {
            orientation: self.orientation.clone(),
        })
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
        if self.state.read().unwrap().layout_mode {
            if off && let Some(idx) = self.handle_layout_key(key) {
                send(KeyData::Layout { idx })?;
            }
        } else if Self::is_note_key(key) {
            send(KeyData::Note {
                position: Self::key_to_coordinate(key, *self.orientation.read().unwrap()),
            })?;
        } else if key.is_multiple_of(20)
            && let Ok(cmd) = CommandKey::try_from(key / 20)
        {
            match cmd {
                CommandKey::Reset => send(KeyData::Reset)?,
                CommandKey::OctaveUp => send(KeyData::OctaveShift { up: true })?,
                CommandKey::OctaveDown => send(KeyData::OctaveShift { up: false })?,
                CommandKey::Shift => send(KeyData::Shift)?,
                CommandKey::Transpose => send(KeyData::Transpose)?,
                CommandKey::Sustain => send(KeyData::Sustain)?,
                CommandKey::Layout => {
                    if off {
                        self.enter_layout_mode()?
                    }
                }
            }
        }
        Ok(())
    }

    fn main_event_loop(&self, event: Event) -> anyhow::Result<()> {
        match event {
            Event::Shutdown => return Ok(()),
            Event::SelectLayout(e) => {
                self.state.write().expect("lock").cur_layout = Some(e.idx);
            }
            Event::ToDevice(_) | Event::KeyEvent(_) => {}
            Event::LightEvent(e) => self.handle_light_event(e)?,
            Event::SetLayoutNames(e) => {
                self.state.write().unwrap().layout_names = e.names;
                #[cfg(test)]
                events::send_test_event(&self.events_tx, TestEvent::LayoutsHandled);
            }
            Event::Reset | Event::UpdateNote(_) | Event::PlayNote(_) => {}
            #[cfg(test)]
            Event::TestEngine(_) | Event::TestWeb(_) | Event::TestEvent(_) | Event::TestSync => {}
        }
        Ok(())
    }
}

fn init_key_map() -> HashMap<Orientation, BiMap<u8, Coordinate>> {
    // We support two orientations of the HexBoard. The main one, just called "hexboard",
    // positions the HexBoard in portrait mode with the command keys on the left.
    //
    // Representing row and column numbers with A=1, B=2, etc. and showing each button s RC,
    // here is a diagram for the "hexboard" layout.
    //
    //   NH  NI  NJ  NK  NL  NM  NN  NO  NP
    // MG  MH  MI  MJ  MK  ML  MM  MN  MO  MP
    //   LG  LH  LI  LJ  LK  LL  LM  LN  LO
    // KF  KG  KH  KI  KJ  KK  KL  KM  KN  KO
    //   JF  JG  JH  JI  JJ  JK  JL  JM  JN
    // IE  IF  IG  IH  II  IJ  IK  IL  IM  IN
    //   HE  HF  HG  HH  HI  HJ  HK  HL  HM
    // GD  GE  GF  GG  GH  GI  GJ  GK  GL  GM
    //   FD  FE  FF  FG  FH  FI  FJ  FK  FL
    // EC  ED  EE  EF  EG  EH  EI  EJ  EK  EL
    //   DC  DD  DE  DF  DG  DH  DI  DJ  DK
    // CB  CC  CD  CE  CF  CG  CH  CI  CJ  CK
    //   BB  BC  BD  BE  BF  BG  BH  BI  BJ
    // AA  AB  AC  AD  AE  AF  AG  AH  AI  AJ
    //
    // We also have a keyboard called "hexboard-60" where the row is 60-degrees from horizontal.
    // In this case, the top "row" would be the two keys at the upper left, and the bottom "row"
    // would be the single key at the bottom right. This results in the following layout:
    //
    //                                     PM  PN
    //                               OK  OL  OM  ON
    //                         NI  NJ  NK  NL  NM  NN
    //                   MG  MH  MI  MJ  MK  ML  MM  MN
    //             LE  LF  LG  LH  LI  LJ  LK  LL  LM  LN
    //       KC  KD  KE  KF  KG  KH  KI  KJ  KK  KL  KM  KN
    // JA  JB  JC  JD  JE  JF  JG  JH  JI  JJ  JK  JL  JM  JN
    //   IA  IB  IC  ID  IE  IF  IG  IH  II  IJ  IK  IL  IM  IN
    //     HA  HB  HC  HD  HE  HF  HG  HH  HI  HJ  HK  HL  HM  HN
    //       GA  GB  GC  GD  GE  GF  GG  GH  GI  GJ  GK  GL  GM
    //         FA  FB  FC  FD  FE  FF  FG  FH  FI  FJ  GK
    //           EA  EB  EC  ED  EE  EF  EG  EH  EI
    //             DA  DB  DC  DD  DE  DF  DG
    //               CA  CB  CC  CD  CE
    //                 BA  BB  BC
    //                   AA
    //
    // If you look at these diagrams, you can see the alternating row pattern 9, 10, 9, 10, ...
    // in both. In the first one, it's just from top to bottom. On the second one, it's from right
    // to left.
    //
    // Keys on the HexBoard are numbered from 0 to 139, left to right, top to bottom. The multiples
    // of 20 are command keys and correspond do the "missing" keys in the 9-column rows. The logic
    // below can be verified with the diagram. Rows are numbered from 1 starting at the bottom.

    // We will populate in key order. The first note key is 1. Key 0 is a command key. That means
    // we will go high to low for rows and low to high for columns.
    let mut key = 1u8;
    let mut horiz: BiMap<u8, Coordinate> = BiMap::with_capacity(133);
    // Row 14, the topmost row, starts with column 8.
    let mut start_col = 8;
    for row in (1..=14).rev() {
        if row % 2 == 1 {
            // Each odd-numbered row starts with a column one lower than the previous row.
            start_col -= 1;
        }
        // Even rows have 9 columns; odd have 10.
        let num_cols = 9 + row % 2;
        for col in start_col..start_col + num_cols {
            horiz.insert(key, Coordinate { row, col });
            key += 1;
            if key.is_multiple_of(20) {
                // Skip command keys.
                key += 1;
            }
        }
    }
    debug_assert_eq!(horiz.len(), 133);
    debug_assert!(horiz.get_by_left(&0).is_none());
    debug_assert!(horiz.get_by_left(&120).is_none());
    debug_assert_eq!(
        horiz.get_by_left(&1).unwrap(),
        &Coordinate { row: 14, col: 8 }
    );
    debug_assert_eq!(
        horiz.get_by_left(&9).unwrap(),
        &Coordinate { row: 14, col: 16 }
    );
    debug_assert_eq!(
        horiz.get_by_left(&10).unwrap(),
        &Coordinate { row: 13, col: 7 }
    );
    debug_assert_eq!(
        horiz.get_by_left(&21).unwrap(),
        &Coordinate { row: 12, col: 7 }
    );
    debug_assert_eq!(
        horiz.get_by_left(&130).unwrap(),
        &Coordinate { row: 1, col: 1 }
    );
    debug_assert_eq!(
        horiz.get_by_left(&139).unwrap(),
        &Coordinate { row: 1, col: 10 }
    );

    // To construct r60, we will operate with HexBoard rows, so the logic corresponds to traversing
    // the diagram from right to left. The right diagonal that goes down to the right corresponds
    // to a column in the layout. You can see in the diagram that the right, down-facing diagonal
    // goes from PN to HN, which is column 14, rows 16 down to 8. We will traverse that way to
    // populate the map in key order.
    let mut r60: BiMap<u8, Coordinate> = BiMap::with_capacity(133);
    let mut start_row = 16;
    key = 1u8;
    for col in (1..=14).rev() {
        // Odd columns have rows have 10 rows; even have 9.
        let num_rows = 9 + col % 2;
        for row in (start_row - num_rows + 1..=start_row).rev() {
            r60.insert(key, Coordinate { row, col });
            key += 1;
            if key.is_multiple_of(20) {
                key += 1;
            }
        }
        if col % 2 == 1 {
            // After an odd column, adjust the starting row for the next column.
            start_row -= 1;
        }
    }
    debug_assert_eq!(r60.len(), 133);
    debug_assert!(r60.get_by_left(&0).is_none());
    debug_assert!(r60.get_by_left(&120).is_none());
    debug_assert_eq!(
        r60.get_by_left(&1).unwrap(),
        &Coordinate { row: 16, col: 14 }
    );
    debug_assert_eq!(
        r60.get_by_left(&9).unwrap(),
        &Coordinate { row: 8, col: 14 }
    );
    debug_assert_eq!(
        r60.get_by_left(&10).unwrap(),
        &Coordinate { row: 16, col: 13 }
    );
    debug_assert_eq!(
        r60.get_by_left(&21).unwrap(),
        &Coordinate { row: 15, col: 12 }
    );
    debug_assert_eq!(
        r60.get_by_left(&130).unwrap(),
        &Coordinate { row: 10, col: 1 }
    );
    debug_assert_eq!(
        r60.get_by_left(&139).unwrap(),
        &Coordinate { row: 1, col: 1 }
    );

    [(Orientation::Horiz, horiz), (Orientation::R60, r60)]
        .into_iter()
        .collect()
}

pub fn hexboard_color(color: Color) -> HSV {
    let hsv = |h, s, v| HSV {
        hue: h,
        sat: s,
        val: v,
    };
    match color {
        // See misc/hexboard-scripts/colors
        Color::Off => hsv(0, 0, 0),
        Color::Active => hsv(0, 0, 127),           // white
        Color::ToggleOff => hsv(0, 127, 127),      // red
        Color::ToggleOn => hsv(50, 127, 127),      // green
        Color::FifthOff => hsv(85, 127, 127),      // blue
        Color::FifthOn => hsv(50, 127, 127),       // green
        Color::MajorThirdOff => hsv(98, 127, 127), // purple
        Color::MajorThirdOn => hsv(116, 72, 127),  // pink
        Color::MinorThirdOff => hsv(0, 127, 127),  // red
        Color::MinorThirdOn => hsv(14, 127, 127),  // orange
        Color::TonicOff => hsv(64, 127, 127),      // cyan
        Color::TonicOn => hsv(28, 127, 127),       // yellow
        Color::OtherOff => hsv(0, 0, 64),          // dull gray
        Color::OtherOn => hsv(0, 0, 127),          // white
        Color::SingleStepOff => hsv(64, 32, 92),   // grayish cyan
        Color::SingleStepOn => hsv(0, 0, 127),     // white
        Color::NoteSelected => hsv(108, 127, 127), // magenta
        Color::LogoBackground => hsv(116, 32, 80), // light pink
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine;
    use crate::events::{LayoutNamesEvent, TestEvent};
    use crate::test_util::TestController;

    #[tokio::test]
    async fn test_select_layout() -> anyhow::Result<()> {
        let mut tc = TestController::new().await;
        let events_tx = tc.tx().downgrade();
        let events_rx = tc.rx();
        let tx = events_tx.upgrade().unwrap();
        let hexboard = Arc::new(HexBoard::new(events_tx));
        engine::start_keyboard(None, hexboard.clone(), events_rx).await?;
        let layout_names: Vec<_> = (0..22).map(|x| x.to_string()).collect();
        tx.send(Event::SetLayoutNames(LayoutNamesEvent {
            names: layout_names.clone(),
        }))?;
        tc.wait_for_test_event(TestEvent::LayoutsHandled).await;
        assert!(!hexboard.state.read().unwrap().layout_mode);
        assert_eq!(hexboard.state.read().unwrap().layout_names.len(), 22);
        let layout_key = 20 * u8::from(CommandKey::Layout);

        // Enter layout mode
        hexboard.handle_raw_event(FromDevice::Key(RawKeyEvent {
            key: layout_key,
            velocity: 0,
        }))?;
        tc.wait_for_test_event(TestEvent::LayoutsHandled).await;
        assert!(hexboard.state.read().unwrap().layout_mode);

        // Cancel layout mode
        hexboard.handle_raw_event(FromDevice::Key(RawKeyEvent {
            key: layout_key,
            velocity: 0,
        }))?;
        tc.wait_for_test_event(TestEvent::LayoutsHandled).await;
        assert!(!hexboard.state.read().unwrap().layout_mode);

        // Enter layout mode
        hexboard.handle_raw_event(FromDevice::Key(RawKeyEvent {
            key: layout_key,
            velocity: 0,
        }))?;
        tc.wait_for_test_event(TestEvent::LayoutsHandled).await;
        assert!(hexboard.state.read().unwrap().layout_mode);

        // Out of range key does nothing. We have 22 layouts. Keys 0 and 20 are command keys,
        // so the highest layout key is 21.
        assert_eq!(HexBoard::key_to_layout_idx(22, 1), Some(0));
        assert_eq!(HexBoard::key_to_layout_idx(22, 19), Some(18));
        assert_eq!(HexBoard::key_to_layout_idx(22, 21), Some(19));
        assert_eq!(HexBoard::key_to_layout_idx(22, 23), Some(21));
        assert!(HexBoard::key_to_layout_idx(22, 0).is_none());
        assert!(HexBoard::key_to_layout_idx(22, 20).is_none());
        assert!(HexBoard::key_to_layout_idx(22, 24).is_none());
        hexboard.handle_raw_event(FromDevice::Key(RawKeyEvent {
            key: 24,
            velocity: 0,
        }))?;
        tc.wait_for_test_event(TestEvent::LayoutsHandled).await;
        assert!(hexboard.state.read().unwrap().layout_mode);

        // Select a layout
        hexboard.handle_raw_event(FromDevice::Key(RawKeyEvent {
            key: 3,
            velocity: 0,
        }))?;
        tc.wait_for_test_event(TestEvent::LayoutSelected).await;
        assert!(!hexboard.state.read().unwrap().layout_mode);
        assert_eq!(hexboard.state.read().unwrap().cur_layout.unwrap(), 2);

        // let ts = tc.get_engine_state().await;
        // assert_eq!(ts.layout.unwrap(), 4);
        // // Scroll layout
        // hexboard.handle_raw_event(FromDevice::Key(RawKeyEvent {
        //     key: 19,
        //     velocity: 0,
        // }))?;
        // tc.wait_for_test_event(TestEvent::LayoutsHandled).await;
        // assert_eq!(hexboard.state.read().expect("lock").layout_offset, 8);
        // hexboard.handle_raw_event(FromDevice::Key(RawKeyEvent {
        //     key: 101,
        //     velocity: 0,
        // }))?;
        // tc.wait_for_test_event(TestEvent::LayoutSelected).await;
        // let ts = tc.get_engine_state().await;
        // assert_eq!(ts.layout.unwrap(), 8);
        //
        // hexboard.handle_raw_event(FromDevice::Key(RawKeyEvent {
        //     key: 19,
        //     velocity: 0,
        // }))?;
        // tc.wait_for_test_event(TestEvent::LayoutsHandled).await;
        // assert_eq!(hexboard.state.read().expect("lock").layout_offset, 0);
        // hexboard.handle_raw_event(FromDevice::Key(RawKeyEvent {
        //     key: 102,
        //     velocity: 0,
        // }))?;
        // tc.wait_for_test_event(TestEvent::LayoutSelected).await;
        // let ts = tc.get_engine_state().await;
        // assert_eq!(ts.layout.unwrap(), 1);

        tc.shutdown().await
    }
}
