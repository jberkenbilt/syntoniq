use askama::Template;
use derive_more::Debug as DebugMore;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use syntoniq_common::parsing::{Coordinate, Layout, Layouts, PlacedNote};
use syntoniq_common::pitch::Pitch;
use tokio::sync::broadcast::error::RecvError;
#[cfg(test)]
use tokio::sync::mpsc;
use tokio::sync::{RwLock, broadcast};

// Neutral gray representing a turned-off key
pub const OFF_RGB: &str = "#616161";

#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub enum Color {
    Off,
    ControlOff,
    Active,
    ToggleOff,
    ToggleOn,
    FourthOff,
    FourthOn,
    FifthOff,
    FifthOn,
    MajorThirdOff,
    MajorThirdOn,
    MinorSixthOff,
    MinorSixthOn,
    MinorThirdOff,
    MinorThirdOn,
    MajorSixthOff,
    MajorSixthOn,
    TonicOff,
    TonicOn,
    SingleStepOff,
    SingleStepOn,
    OtherOff,
    OtherOn,
    NoteSelected,
    LogoPink,
    LogoRed,
    LogoGreen,
    LogoBlue,
}

#[derive(Copy, Clone)]
pub struct NoteColors {
    pub off: Color,
    pub on: Color,
}

pub fn interval_color(mut interval: f32) -> NoteColors {
    while interval <= 1.0 {
        interval *= 2.0;
    }
    while interval >= 2.0 {
        interval /= 2.0;
    }
    // If the color is very close to of the 5-limit Just Intonation ratios below or their
    // reciprocals, assign a color. Otherwise, assign a default.
    // Note: 12-EDO minor third is by 15.64 cents.
    let tolerance_cents = 2.0f32.powf(16.0 / 1200.0);
    for (ratio, (off, on)) in [
        (1.0, (Color::TonicOff, Color::TonicOn)),
        (3.0 / 2.0, (Color::FifthOff, Color::FifthOn)),
        (4.0 / 3.0, (Color::FourthOff, Color::FourthOn)),
        (5.0 / 4.0, (Color::MajorThirdOff, Color::MajorThirdOn)),
        (6.0 / 5.0, (Color::MinorThirdOff, Color::MinorThirdOn)),
        (8.0 / 5.0, (Color::MinorSixthOff, Color::MinorSixthOn)),
        (5.0 / 3.0, (Color::MajorSixthOff, Color::MajorSixthOn)),
    ] {
        // Interval will never be zero unless someone put zeros in their scale files, and we
        // check against that when validating the config file.
        let difference = if interval > ratio {
            interval / ratio
        } else {
            ratio / interval
        };
        if difference < tolerance_cents {
            return NoteColors { off, on };
        }
    }
    NoteColors {
        off: Color::OtherOff,
        on: Color::OtherOn,
    }
}

#[derive(Default, Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Hash)]
pub enum Orientation {
    #[default]
    Horiz,
    R60,
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Hash)]
pub enum ButtonData {
    Note {
        position: Coordinate,
        orientation: Option<Orientation>,
    },
    Command {
        idx: u8,
    },
}

#[derive(Clone, Debug)]
pub struct RawLightEvent {
    pub key: u8,
    pub color: Color,
    pub rgb_color: String,
    pub label1: String,
    pub label2: String,
}
#[derive(Clone, Debug)]
pub struct RawKeyEvent {
    /// Midi note number
    pub key: u8,
    /// 0..127, 0 = off
    pub velocity: u8,
}
#[derive(Clone, Debug)]
pub struct RawPressureEvent {
    pub key: Option<u8>,
    pub velocity: u8,
}

#[derive(Clone, Copy, Debug)]
pub enum KeyData {
    Shift,
    Layout { idx: usize },
    Reset,
    Sustain,
    Transpose,
    OctaveShift { up: bool },
    Print,
    Note { position: Coordinate },
}

#[derive(Clone, Debug)]
pub struct KeyEvent {
    pub key: KeyData,
    pub velocity: u8,
}

#[derive(Clone, Debug)]
pub enum LightData {
    Shift,
    Sustain,
    Transpose,
}
#[derive(Clone, Debug)]
pub struct LightEvent {
    pub light: LightData,
    pub color: Color,
    pub label1: String,
    pub label2: String,
}

#[derive(Clone, Debug)]
pub struct LayoutNamesEvent {
    pub names: Vec<String>,
}

#[derive(Clone, DebugMore)]
pub struct SelectLayoutEvent {
    pub idx: usize,
    #[debug("{}", layout.name)]
    pub layout: Arc<Layout<'static>>,
}

#[derive(Clone, Debug)]
pub struct UpdateNoteEvent {
    pub position: Coordinate,
    pub note: Option<Arc<Note>>,
}

#[derive(DebugMore)]
pub struct Note {
    #[debug("placed:name={},scale={}", placed.name, placed.scale_name)]
    pub placed: PlacedNote<'static>,
    pub off_color: Color,
    pub on_color: Color,
}
impl Note {
    fn format_note(&self) -> String {
        let name = self.placed.name.as_ref();
        let tile_factor = &self.placed.tile_factor;
        let base_factor = &self.placed.untiled_base_relative / tile_factor;
        let tile = if tile_factor == &Pitch::unit() {
            String::new()
        } else {
            format!(" × {tile_factor}")
        };
        let pitch = &self.placed.pitch;
        format!("{name} (base × {base_factor}{tile} = {pitch})")
    }

    pub fn format_with_scale(&self) -> String {
        let scale_description = self.format_mapping();
        let note = self.format_note();
        format!("{note}, scale={scale_description}")
    }

    fn format_mapping(&self) -> String {
        let scale_name = self.placed.scale_name.as_ref();
        let orig_base_pitch = &self.placed.scale_base;
        let transposition = &self.placed.transposition;
        let base_pitch = orig_base_pitch * transposition;
        let mut result = format!("{scale_name}, base={base_pitch}");
        if transposition != &Pitch::unit() {
            result.push_str(&format!(
                " (transposition: {orig_base_pitch} × {transposition})"
            ));
        }
        result
    }
}
#[derive(Clone, Debug)]
pub struct PlayNoteEvent {
    pub pitch: Pitch,
    pub velocity: u8,
    pub note: Option<Arc<Note>>,
}

#[derive(Debug, Clone)]
pub struct SpecificNote {
    pub layout_idx: usize,
    pub note: Arc<Note>,
    pub position: Coordinate,
}

#[derive(Default, Clone, DebugMore)]
pub struct EngineState {
    /// Currently selected layout
    pub layout: Option<usize>,
    /// All available layouts
    #[debug(skip)]
    pub layouts: Arc<Layouts<'static>>,
    /// Mapping from position to note in the current layout
    pub notes: HashMap<Coordinate, Option<Arc<Note>>>,
    /// Mapping from pitch to all positions with that pitch in the current layout
    pub pitch_positions: HashMap<Pitch, HashSet<Coordinate>>,
    /// Number of times a pitch is on; > 1 if simultaneously touching more than one position
    /// with the same pitch in non-sustain mode
    pub pitch_on_count: HashMap<Pitch, u8>,
    /// Last note played for a given pitch
    pub last_note_for_pitch: HashMap<Pitch, Arc<Note>>,
    /// Positions that are actually being touched
    pub positions_down: HashMap<Coordinate, Arc<Note>>,
    pub sustain: bool,
    /// When sustain is turned off, this is set. It is cleared on any note event. If sustain is
    /// turned back on without intervening note events within a certain period of time, all notes
    /// are turned off.
    pub last_sustain_off_time: Option<Instant>,
    pub shift: Option<Option<SpecificNote>>,
    pub transpose: Option<Option<SpecificNote>>,
}
impl EngineState {
    pub fn current_layout(&self) -> Option<Arc<Layout<'static>>> {
        self.layout
            .and_then(|x| self.layouts.layouts.get(x).cloned())
    }

    pub fn current_played_notes(&self) -> Vec<String> {
        // It would be more efficient to directly print, but this is not performance-critical,
        // and generating a Vec makes testing easier.
        let mut result = Vec::new();
        // Scale name -> notes in the scale
        let mut scale_to_notes: HashMap<String, Vec<&Note>> = HashMap::new();
        for note in self.last_note_for_pitch.values() {
            let key = note.format_mapping();
            scale_to_notes.entry(key).or_default().push(note.as_ref());
        }
        let mut keys: Vec<String> = scale_to_notes.keys().cloned().collect();
        keys.sort();
        for scale in keys {
            result.push(format!("Scale: {scale}"));
            let mut notes = scale_to_notes.remove(&scale).unwrap();
            notes.sort_by_key(|note| note.placed.pitch.clone());
            for note in notes {
                let formatted = note.format_note();
                result.push(format!("  Note: {formatted}"));
            }
        }
        result
    }
}

#[derive(Template, Default, Clone)]
#[template(path = "state-view.html")]
pub struct StateView {
    pub selected_layout: String,
    pub layout_names: Vec<String>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TestEvent {
    ResetComplete,
    LayoutSelected,
    LayoutsHandled,
    HandledNote,
    HandledKey,
    MoveCanceled,
    Sync,
}

#[derive(Clone, Debug)]
pub enum FromDevice {
    Key(RawKeyEvent),
    Pressure(RawPressureEvent),
}

#[derive(Clone, Debug)]
pub enum ToDevice {
    Light(Vec<RawLightEvent>),
}

#[derive(Clone, Debug)]
pub enum Event {
    Shutdown,
    ToDevice(ToDevice),
    Reset,
    KeyEvent(KeyEvent),
    LightEvent(LightEvent),
    SetLayoutNames(LayoutNamesEvent),
    SelectLayout(SelectLayoutEvent),
    UpdateNote(UpdateNoteEvent),
    PlayNote(PlayNoteEvent),
    #[cfg(test)]
    TestEngine(mpsc::Sender<EngineState>),
    #[cfg(test)]
    TestWeb(mpsc::Sender<StateView>),
    #[cfg(test)]
    TestEvent(TestEvent),
    #[cfg(test)]
    TestSync,
}

pub type UpgradedSender = broadcast::Sender<Event>;
pub type WeakSender = broadcast::WeakSender<Event>;
pub type Receiver = broadcast::Receiver<Event>;

pub struct Events {
    tx: RwLock<Option<UpgradedSender>>,
    rx: Receiver,
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub fn send_test_event(events_tx: &WeakSender, test_event: TestEvent) {
    if let Some(tx) = events_tx.upgrade() {
        tx.send(Event::TestEvent(test_event)).unwrap();
    }
}

/// Receive an event, ignoring lag
pub async fn receive_check_lag(rx: &mut Receiver, warn_prefix: Option<&str>) -> Option<Event> {
    loop {
        let event = rx.recv().await;
        match event {
            Ok(Event::Shutdown) => return None,
            Ok(event) => return Some(event),
            Err(err) => match err {
                RecvError::Closed => return None,
                RecvError::Lagged(n) => {
                    if let Some(p) = warn_prefix {
                        log::warn!("{p}: missed {n} events");
                    }
                    continue;
                }
            },
        }
    }
}

impl Events {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(1000);
        Self {
            tx: RwLock::new(Some(tx)),
            rx,
        }
    }

    pub async fn sender(&self) -> WeakSender {
        let tx = self
            .tx
            .read()
            .await
            .clone()
            .expect("sender called after shutdown");
        tx.downgrade()
    }

    pub fn receiver(&self) -> Receiver {
        self.rx.resubscribe()
    }

    pub async fn shutdown(&self) {
        self.tx.write().await.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_colors() {
        fn get_color(pitch: &str) -> Color {
            let NoteColors { off, .. } = interval_color(Pitch::must_parse(pitch).as_float());
            off
        }
        assert_eq!(get_color("3/2"), Color::FifthOff); // JI 5th
        assert_eq!(get_color("^9|12"), Color::MajorSixthOff); // 12-EDO major sixth
        assert_eq!(get_color("^10|31"), Color::MajorThirdOff); // 31-EDO major third
        assert_eq!(get_color("^39|31"), Color::MinorThirdOff); // 31-EDO minor third
        assert_eq!(get_color("2*^21|31"), Color::MinorSixthOff); // 31-EDO minor sixth
        assert_eq!(get_color("1/2*^10|31"), Color::MajorThirdOff); // 31-EDO major third
        assert_eq!(get_color("^7|17"), Color::FourthOff); // 17-EDO fourth
        assert_eq!(get_color("^5|17"), Color::OtherOff); // nope
    }
}
