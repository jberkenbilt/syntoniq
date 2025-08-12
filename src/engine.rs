use crate::config::Config;
#[cfg(test)]
use crate::events::{AugmentedEngineState, TestEvent};
use crate::events::{
    Color, EngineState, Event, KeyEvent, LayoutNamesEvent, LightEvent, LightMode, PlayNoteEvent,
    SelectLayoutEvent, ShiftKeyState, ShiftLayoutState, SpecificNote, TransposeState,
    UpdateNoteEvent,
};
use crate::layout::Layout;
use crate::pitch::{Factor, Pitch};
use crate::scale::{Note, ScaleType};
use crate::{controller, csound, events, midi_player};
use anyhow::{anyhow, bail};
use chrono::SubsecRound;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(test)]
mod tests;

mod keys {
    // Top Row, left to right
    pub const SHIFT: u8 = 90;
    pub const TRANSPOSE: u8 = 94; // Note
    pub const SUSTAIN: u8 = 95; // Chord
    // Left column, top to bottom
    pub const UP_ARROW: u8 = 80;
    pub const DOWN_ARROW: u8 = 70;
    pub const CLEAR: u8 = 60;
    pub const RECORD: u8 = 10;
    // Right column, top to bottom
    pub const LAYOUT_SCROLL: u8 = 19;
    // Upper bottom controls
    pub const LAYOUT_MIN: u8 = 101;
    pub const LAYOUT_MAX: u8 = 109;
}

#[derive(Debug)]
pub enum SoundType {
    None,
    Midi,
    Csound,
}

#[derive(Debug, Clone)]
pub struct PlayedNote {
    pub note: Arc<Note>,
    pub velocity: u8,
}

struct Engine {
    config_file: PathBuf,
    events_tx: events::WeakSender,
    layouts: Vec<Arc<RwLock<Layout>>>,
    transient_state: EngineState,
}

impl Engine {
    fn layout_at_position(&self, position: u8) -> Option<(usize, Arc<RwLock<Layout>>)> {
        let idx = (position - keys::LAYOUT_MIN) as usize + self.transient_state.layout_offset;
        let layout = self.layouts.get(idx).cloned();
        layout.map(|x| (idx, x))
    }

    fn current_layout(&self) -> Option<Arc<RwLock<Layout>>> {
        self.transient_state
            .layout
            .and_then(|x| self.layouts.get(x).cloned())
    }

    async fn reset(&mut self) -> anyhow::Result<()> {
        let config =
            Config::load(&self.config_file).map_err(|e| anyhow!("error reloading config: {e}"))?;
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        self.layouts = config.layouts;
        let mut names = Vec::new();
        for layout in &self.layouts {
            names.push(layout.read().await.name.clone());
        }
        tx.send(Event::SetLayoutNames(LayoutNamesEvent { names }))?;

        // Turn off all notes
        for (pitch, count) in &self.transient_state.pitch_on_count {
            if *count > 0 {
                tx.send(Event::PlayNote(PlayNoteEvent {
                    pitch: pitch.clone(),
                    velocity: 0,
                    note: None,
                }))?;
            }
        }
        self.transient_state = Default::default();

        // Draw the logo.
        controller::clear_lights(&tx).await?;
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
                tx.send(Event::Light(LightEvent {
                    mode: LightMode::On,
                    position,
                    color,
                    label1: String::new(),
                    label2: String::new(),
                }))?;
            }
        }
        for (position, label1, label2) in [
            (keys::UP_ARROW, "▲", ""),
            (keys::DOWN_ARROW, "▼", ""),
            (keys::CLEAR, "Reset", ""),
            (keys::RECORD, "Show", "Notes"),
        ] {
            tx.send(Event::Light(LightEvent {
                mode: LightMode::On,
                position,
                color: Color::Active,
                label1: label1.to_string(),
                label2: label2.to_string(),
            }))?;
        }
        self.fix_layout_lights(&tx).await?;
        #[cfg(test)]
        self.send_test_event(TestEvent::ResetComplete);
        log::info!("QLaunchPad is initialized");
        Ok(())
    }

    async fn fix_layout_lights(&mut self, tx: &events::UpgradedSender) -> anyhow::Result<()> {
        for i in 0..=8 {
            let position = keys::LAYOUT_MIN + i;
            let idx = i as usize + self.transient_state.layout_offset;
            let event = if idx < self.layouts.len() {
                let is_cur = self
                    .transient_state
                    .layout
                    .map(|x| x == idx)
                    .unwrap_or(false);
                LightEvent {
                    mode: LightMode::On,
                    position,
                    color: if is_cur {
                        Color::ToggleOn
                    } else {
                        Color::Active
                    },
                    label1: (idx + 1).to_string(),
                    label2: String::new(),
                }
            } else {
                LightEvent {
                    mode: LightMode::Off,
                    position,
                    color: Color::Off,
                    label1: String::new(),
                    label2: String::new(),
                }
            };
            tx.send(Event::Light(event))?;
        }
        if self.layouts.len() > 8 {
            tx.send(Event::Light(LightEvent {
                mode: LightMode::On,
                position: keys::LAYOUT_SCROLL,
                color: Color::Active,
                label1: "Scroll".to_string(),
                label2: "layouts".to_string(),
            }))?;
        }
        Ok(())
    }

    async fn handle_key(&mut self, key_event: KeyEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let KeyEvent { key, velocity, .. } = key_event;
        let off = velocity == 0;
        let have_layout = self.transient_state.layout.is_some();
        if !off && matches!(self.transient_state.shift_key_state, ShiftKeyState::Down) {
            // Update shift state -- see below for behavior of shift key.
            self.set_shift(ShiftKeyState::On, &tx)?;
        }
        match key {
            keys::SHIFT if have_layout => {
                // Behavior of shift key:
                // - When pressed, state transitions from Off to Down
                // - If any other key is pressed while in Down state, state transitions to On
                // - If shift is released when Down, it changes to On
                // - If shift is pressed or released when On, it changes to Off
                // Effect: pressing and releasing the shift key without touching other keys toggles
                // its state. Touching another key while holding it makes it act like a modifier.
                let shift = match (self.transient_state.shift_key_state, off) {
                    (ShiftKeyState::Off, false) => ShiftKeyState::Down,
                    (ShiftKeyState::Down, true) => ShiftKeyState::On,
                    (ShiftKeyState::On, _) => ShiftKeyState::Off,
                    _ => self.transient_state.shift_key_state,
                };
                self.set_shift(shift, &tx)?;
            }
            keys::LAYOUT_MIN..=keys::LAYOUT_MAX if off => {
                if let Some((idx, layout)) = self.layout_at_position(key) {
                    tx.send(Event::SelectLayout(SelectLayoutEvent { idx, layout }))?;
                }
            }
            keys::LAYOUT_SCROLL if off => {
                tx.send(Event::ScrollLayouts)?;
            }
            keys::CLEAR if off => {
                tx.send(Event::Reset)?;
            }
            keys::SUSTAIN if off => {
                self.transient_state.sustain = !self.transient_state.sustain;
                tx.send(self.sustain_light_event())?;
            }
            keys::TRANSPOSE if off && have_layout => {
                self.transient_state.transpose_state = match self.transient_state.transpose_state {
                    TransposeState::Off => TransposeState::Pending {
                        initial_layout: self.transient_state.layout.unwrap(),
                    },
                    _ => TransposeState::Off,
                };
                tx.send(self.transpose_light_event())?;
            }
            keys::UP_ARROW | keys::DOWN_ARROW if off && have_layout => {
                // 2025-07-22, rust 1.88: "if let guards" are experimental. When stable, we can
                // use one instead of is_some above and get rid of this unwrap.
                let layout = self.current_layout().unwrap();
                {
                    let locked = &mut *layout.write().await;
                    let transposition = if key == keys::UP_ARROW {
                        Pitch::new(vec![Factor::new(2, 1, 1, 1)?])
                    } else {
                        Pitch::new(vec![Factor::new(1, 2, 1, 1)?])
                    };
                    locked.scale.transpose(&transposition);
                }
                tx.send(Event::SelectLayout(SelectLayoutEvent {
                    idx: self.transient_state.layout.unwrap(),
                    layout,
                }))?;
            }
            keys::RECORD if off && have_layout => {
                self.print_notes();
            }
            position if have_layout && self.transient_state.notes.contains_key(&position) => {
                if let Some(note) = self.transient_state.notes.get(&position).unwrap() {
                    self.handle_note_key(&tx, note.clone(), position, off)
                        .await?;
                };
            }
            _ => (),
        }
        #[cfg(test)]
        self.send_test_event(TestEvent::HandledKey);
        Ok(())
    }

    fn print_notes(&self) {
        println!(
            "----- Current Notes ({}) -----",
            chrono::offset::Local::now().trunc_subsecs(0)
        );
        for s in self.transient_state.current_played_notes() {
            println!("{s}");
        }
    }

    async fn handle_note_key(
        &mut self,
        tx: &events::UpgradedSender,
        note: Arc<Note>,
        position: u8,
        off: bool,
    ) -> anyhow::Result<()> {
        let is_transpose = !matches!(self.transient_state.transpose_state, TransposeState::Off);
        let is_shift = !matches!(self.transient_state.shift_key_state, ShiftKeyState::Off);
        let mut play_note = !is_transpose && !is_shift;
        let layout = self
            .transient_state
            .layout
            .expect("handle_note_key called without current layout");
        if is_transpose {
            let note = note.clone();
            match self.transient_state.transpose_state.clone() {
                TransposeState::Off => unreachable!(),
                TransposeState::Pending { initial_layout } => {
                    if !off {
                        self.transient_state.transpose_state = TransposeState::FirstSelected {
                            initial_layout,
                            note1: SpecificNote {
                                layout_idx: layout,
                                note,
                                position,
                            },
                        };
                    }
                }
                TransposeState::FirstSelected {
                    initial_layout,
                    note1,
                } => {
                    if !off {
                        self.handle_transpose(
                            initial_layout,
                            note1,
                            SpecificNote {
                                layout_idx: layout,
                                note,
                                position,
                            },
                        )
                        .await?;
                    }
                }
            }
        } else if is_shift && !off {
            self.handle_shift(
                layout,
                SpecificNote {
                    layout_idx: layout,
                    note: note.clone(),
                    position,
                },
            )
            .await?;
        }
        if is_transpose
            && matches!(
                self.transient_state.transpose_state,
                TransposeState::FirstSelected { .. }
            )
        {
            play_note = true;
        }
        if play_note {
            self.handle_note_key_normal(tx, note, position, off)?;
        }

        if is_transpose && let Some(tx) = self.events_tx.upgrade() {
            tx.send(self.transpose_light_event())?;
        }

        #[cfg(test)]
        self.send_test_event(TestEvent::HandledNote);
        Ok(())
    }

    async fn handle_shift(&mut self, layout_idx: usize, note: SpecificNote) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let mut update_layout = false;
        match self.transient_state.shift_layout_state.clone() {
            ShiftLayoutState::Off => {
                self.transient_state.shift_layout_state = ShiftLayoutState::FirstSelected(note);
            }
            ShiftLayoutState::FirstSelected(note1) => {
                self.transient_state.shift_layout_state = ShiftLayoutState::Off;
                if note1.layout_idx != layout_idx {
                    log::info!("move: note1 and note2 are from different layouts, so not shifting");
                    #[cfg(test)]
                    self.send_test_event(TestEvent::MoveCanceled);
                } else {
                    let mut layout = self.layouts[layout_idx].write().await;
                    if let Some(base) = layout.base {
                        let note1_col = note1.position % 10;
                        let note1_row = note1.position / 10;
                        let note2_col = note.position % 10;
                        let note2_row = note.position / 10;
                        let dy = note2_row as i8 - note1_row as i8;
                        let dx = note2_col as i8 - note1_col as i8;
                        log::info!("shifting layout {} by dy={dy}, dx={dx}", layout.name);
                        let (old_x, old_y) = base;
                        layout.base = Some((old_x + dx, old_y + dy));
                        update_layout = true;
                    } else {
                        log::info!("move: can't shift non-EDO layout");
                        #[cfg(test)]
                        self.send_test_event(TestEvent::MoveCanceled);
                    };
                }
            }
        };

        if update_layout {
            tx.send(Event::SelectLayout(SelectLayoutEvent {
                idx: layout_idx,
                layout: self.layouts[layout_idx].clone(),
            }))?;
        }
        Ok(())
    }

    async fn handle_transpose(
        &mut self,
        initial_layout: usize,
        note1: SpecificNote,
        note2: SpecificNote,
    ) -> anyhow::Result<()> {
        let mut update_layout = false;
        if note1.note == note2.note {
            self.transient_state.transpose_state = TransposeState::Off;
            let mut layout = self.layouts[initial_layout].write().await;
            log::info!(
                "resetting base pitch of {} to {}",
                layout.scale.name,
                note1.note.pitch
            );
            layout.scale.base_pitch = note1.note.pitch.clone();
            update_layout = true;
        } else {
            // Reset note1 to current note
            self.transient_state.transpose_state = TransposeState::FirstSelected {
                initial_layout,
                note1: note2,
            };
        }
        if update_layout && let Some(tx) = self.events_tx.upgrade() {
            tx.send(Event::SelectLayout(SelectLayoutEvent {
                idx: initial_layout,
                layout: self.layouts[initial_layout].clone(),
            }))?;
        }
        Ok(())
    }

    fn handle_note_key_normal(
        &mut self,
        tx: &events::UpgradedSender,
        note: Arc<Note>,
        position: u8,
        off: bool,
    ) -> anyhow::Result<()> {
        let pitch = &note.pitch;
        let Some(others) = self.transient_state.pitch_positions.get(pitch) else {
            // This would indicate a bug in which we assigned something to notes
            // without also assigning its position to note positions or otherwise
            // allowed notes and note_positions to get out of sync.
            bail!("note positions is missing for {pitch}");
        };
        if off {
            let old = self.transient_state.positions_down.remove(&position);
            if old.is_none() {
                // We got key off event on a note square, but the note was not on. This happens
                // if you touch a key that has no note, and while continuing to touch the key,
                // select a new layout where that key does have a note. Just ignore the event.
                return Ok(());
            }
        } else {
            self.transient_state
                .positions_down
                .insert(position, note.clone());
        }

        let pitch_count = self
            .transient_state
            .pitch_on_count
            .entry(pitch.clone())
            .or_default();
        // When not in sustain mode, touch turns a note on, and release turns it off.
        // Since the same note may appear in multiple locations, we keep a count, and on
        // send a note event if we transition to or from 0. In sustain mode, "off"
        // events are ignored. Touching a note in any of its positions toggles whether
        // it's on or off. Changing scales, transposing, shifting, etc. doesn't affect
        // which notes are on or off, making it possible to play a note in one scale,
        // switch scales, and play a note in another scale.
        let was_on = *pitch_count > 0;
        if self.transient_state.sustain {
            if !off {
                if *pitch_count > 0 {
                    *pitch_count = 0;
                } else {
                    *pitch_count = 1;
                }
            }
        } else if off {
            if *pitch_count > 0 {
                *pitch_count -= 1
            }
        } else {
            *pitch_count += 1;
        }
        let pitch_count = *pitch_count;
        if pitch_count == 0 {
            self.transient_state.pitch_on_count.remove(pitch);
            self.transient_state.last_note_for_pitch.remove(pitch);
        } else {
            self.transient_state
                .last_note_for_pitch
                .insert(pitch.clone(), note.clone());
        }
        let is_on = pitch_count > 0;
        if is_on != was_on {
            let velocity = if is_on { 127 } else { 0 };
            for position in others.iter().copied() {
                tx.send(note.light_event(position, velocity))?;
            }
            tx.send(Event::PlayNote(PlayNoteEvent {
                pitch: pitch.clone(),
                velocity,
                note: Some(note.clone()),
            }))?;
        }
        Ok(())
    }

    async fn update_note(&mut self, event: UpdateNoteEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let UpdateNoteEvent {
            position,
            played_note,
        } = event;
        self.transient_state
            .notes
            .insert(position, played_note.as_ref().map(|x| x.note.clone()));
        match played_note {
            Some(played_note) => {
                let note = played_note.note.clone();
                self.transient_state
                    .pitch_positions
                    .entry(note.pitch.clone())
                    .or_default()
                    .insert(position);
                tx.send(note.light_event(position, played_note.velocity))?;
            }
            None => {
                tx.send(Event::Light(LightEvent {
                    mode: LightMode::Off,
                    position,
                    color: Color::Off,
                    label1: String::new(),
                    label2: String::new(),
                }))?;
            }
        }
        Ok(())
    }

    async fn handle_play_note(&mut self, e: PlayNoteEvent) -> anyhow::Result<()> {
        if let Some(note) = e.note
            && e.velocity > 0
        {
            println!("{note}");
        }
        Ok(())
    }

    fn send_note(
        &self,
        tx: &events::UpgradedSender,
        row: i8,
        col: i8,
        note: Arc<Note>,
    ) -> anyhow::Result<()> {
        let velocity = if self
            .transient_state
            .pitch_on_count
            .get(&note.pitch)
            .copied()
            .unwrap_or_default()
            > 0
        {
            127
        } else {
            0
        };
        let played_note = Some(PlayedNote { note, velocity });
        let position = (10 * row + col) as u8;
        tx.send(Event::UpdateNote(UpdateNoteEvent {
            position,
            played_note,
        }))?;
        Ok(())
    }

    async fn draw_edo_layout(&mut self, layout: &mut Layout) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let ScaleType::EqualDivision(ed) = &layout.scale.scale_type else {
            // Should not be possible
            bail!("draw_edo_layout called with non-EDO scale");
        };
        let (steps_x, steps_y) = layout.steps.unwrap(); // checked to be Some in config
        let (base_x, base_y) = layout.base.unwrap(); // checked to be Some in config
        let (divisions, _, _) = ed.divisions;
        let divisions = divisions as i32;
        for row in 1..=8 {
            for col in 1..=8 {
                let steps = (steps_x * (col - base_x) + steps_y * (row - base_y)) as i32;
                let cycle = steps / divisions;
                let step = steps % divisions;
                let note = layout.scale.edo_note(cycle as i8, step as i8);
                self.send_note(&tx, row, col, Arc::new(note))?;
            }
        }
        Ok(())
    }

    async fn draw_generic_layout(&mut self, layout: &mut Layout) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let ScaleType::Generic(g) = &layout.scale.scale_type else {
            // Should not be possible
            bail!("draw_generic_layout called with non-Generic scale");
        };
        let mut cache = HashMap::new();
        for row in 1..=8 {
            for col in 1..=8 {
                if let Some(note) = layout.scale.generic_note(&mut cache, g, row, col)? {
                    self.send_note(&tx, row, col, note)?;
                } else {
                    tx.send(Event::Light(LightEvent {
                        mode: LightMode::Off,
                        position: (10 * row + col) as u8,
                        color: Color::Off,
                        label1: "".to_string(),
                        label2: "".to_string(),
                    }))?;
                }
            }
        }
        Ok(())
    }

    fn toggle_light_event(&self, on: bool, position: u8, label1: &str, label2: &str) -> Event {
        let color = if on {
            Color::ToggleOn
        } else {
            Color::ToggleOff
        };
        Event::Light(LightEvent {
            mode: LightMode::On,
            position,
            color,
            label1: label1.to_string(),
            label2: label2.to_string(),
        })
    }

    fn transpose_light_event(&self) -> Event {
        let color = match self.transient_state.transpose_state {
            TransposeState::Off => Color::ToggleOff,
            TransposeState::Pending { .. } => Color::ToggleOn,
            TransposeState::FirstSelected { .. } => Color::NoteSelected,
        };
        Event::Light(LightEvent {
            mode: LightMode::On,
            position: keys::TRANSPOSE,
            color,
            label1: "Transpose".to_string(),
            label2: String::new(),
        })
    }

    fn sustain_light_event(&self) -> Event {
        self.toggle_light_event(self.transient_state.sustain, keys::SUSTAIN, "Sustain", "")
    }

    fn set_shift(
        &mut self,
        shift: ShiftKeyState,
        tx: &events::UpgradedSender,
    ) -> anyhow::Result<()> {
        if shift == self.transient_state.shift_key_state {
            return Ok(());
        }
        self.transient_state.shift_key_state = shift;
        if matches!(shift, ShiftKeyState::Off) {
            self.transient_state.shift_layout_state = ShiftLayoutState::Off;
        }
        tx.send(self.shift_light_event())?;
        Ok(())
    }

    fn shift_light_event(&self) -> Event {
        let color = match self.transient_state.shift_key_state {
            ShiftKeyState::Off => Color::ToggleOff,
            _ => Color::ToggleOn,
        };
        Event::Light(LightEvent {
            mode: LightMode::On,
            position: keys::SHIFT,
            color,
            label1: "Shift".to_string(),
            label2: String::new(),
        })
    }

    async fn select_layout(&mut self, event: SelectLayoutEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        // For any keys that are held down, act like we released them. We will send new key events
        // at the end. This creates better behavior if you select a new layout (including octave
        // shift) while holding keys down.
        let notes_down = self.transient_state.positions_down.clone();
        let note_positions_before: HashSet<u8> = notes_down.keys().copied().collect();
        for (position, note) in notes_down {
            self.handle_note_key_normal(&tx, note, position, true)?;
        }
        let layout_lock = event.layout;
        self.transient_state.layout = Some(event.idx);
        let layout = &mut *layout_lock.write().await;
        self.transient_state.pitch_positions.clear();
        self.transient_state.notes.clear();
        match layout.scale.scale_type {
            ScaleType::EqualDivision(_) => self.draw_edo_layout(layout).await?,
            ScaleType::Generic(_) => self.draw_generic_layout(layout).await?,
        }
        log::info!(
            "selected layout: {}, scale: {}",
            layout.name,
            layout.scale.name
        );
        self.fix_layout_lights(&tx).await?;
        tx.send(self.sustain_light_event())?;
        tx.send(self.transpose_light_event())?;
        tx.send(self.shift_light_event())?;
        // Re-touch all the notes that we previously untouched. We do this with synthetic key
        // events because the positions may or may not still have notes.
        for key in note_positions_before {
            tx.send(Event::Key(KeyEvent {
                key,
                velocity: 127,
                synthetic: true,
            }))?;
        }
        #[cfg(test)]
        self.send_test_event(TestEvent::LayoutSelected);
        Ok(())
    }

    async fn scroll_layouts(&mut self) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        self.transient_state.layout_offset += 8;
        if self.transient_state.layout_offset >= self.layouts.len() {
            self.transient_state.layout_offset = 0;
        }
        self.fix_layout_lights(&tx).await?;
        #[cfg(test)]
        self.send_test_event(TestEvent::LayoutsScrolled);
        Ok(())
    }

    async fn handle_event(&mut self, event: Event) -> anyhow::Result<bool> {
        match event {
            Event::Shutdown => return Ok(true),
            Event::Light(_) => {}
            Event::Key(e) => self.handle_key(e).await?,
            Event::Pressure(_) => {}
            Event::Reset => self.reset().await?,
            Event::SelectLayout(e) => self.select_layout(e).await?,
            Event::ScrollLayouts => self.scroll_layouts().await?,
            Event::SetLayoutNames(_) => {}
            Event::UpdateNote(e) => self.update_note(e).await?,
            Event::PlayNote(e) => self.handle_play_note(e).await?,
            #[cfg(test)]
            Event::TestEngine(test_tx) => {
                test_tx
                    .send(AugmentedEngineState {
                        engine_state: self.transient_state.clone(),
                        layout: self.current_layout(),
                    })
                    .await?
            }
            #[cfg(test)]
            Event::TestWeb(_) => {}
            #[cfg(test)]
            Event::TestEvent(_) => {}
            #[cfg(test)]
            Event::TestSync => self.send_test_event(TestEvent::Sync),
        };
        Ok(false)
    }

    #[cfg(test)]
    fn send_test_event(&self, test_event: TestEvent) {
        if let Some(tx) = self.events_tx.upgrade() {
            tx.send(Event::TestEvent(test_event)).unwrap();
        }
    }
}

pub async fn run(
    config_file: PathBuf,
    sound_type: SoundType,
    events_tx: events::WeakSender,
    mut events_rx: events::Receiver,
) -> anyhow::Result<()> {
    let mut engine = Engine {
        config_file,
        events_tx: events_tx.clone(),
        layouts: Default::default(),
        transient_state: Default::default(),
    };
    let rx2 = events_rx.resubscribe();
    let tx2 = events_tx.clone();
    match sound_type {
        SoundType::None => {}
        SoundType::Midi => {
            tokio::spawn(async move {
                if let Err(e) = midi_player::play_midi(rx2).await {
                    log::error!("midi player error: {e}");
                };
            });
        }
        SoundType::Csound => {
            tokio::spawn(async move {
                if let Err(e) = csound::run_csound(rx2, tx2).await {
                    log::error!("csound player error: {e}");
                };
            });
        }
    }
    if let Some(tx) = events_tx.upgrade() {
        tx.send(Event::Reset)?;
    }
    while let Some(event) = events::receive_check_lag(&mut events_rx, Some("engine")).await {
        // Note: this event loop calls event handlers inline. Sometimes those event handlers
        // generate other events, which are piling up in our queue while we are handling earlier
        // events. As long as the backlog on the event receiver is high enough and/or we don't
        // care about missing some messages, we should be fine. In practice, the messages we would
        // be most likely to miss our the ones we just sent, but we could also miss other people's
        // responses to those. We are not as likely to miss user events.
        match engine.handle_event(event).await {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(e) => {
                log::error!("error handling event: {e}");
            }
        };
    }
    Ok(())
}
