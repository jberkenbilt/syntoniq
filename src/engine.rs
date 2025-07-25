use crate::config::Config;
#[cfg(test)]
use crate::events::TestEvent;
use crate::events::{
    AssignLayoutEvent, Color, EngineState, Event, KeyEvent, LightEvent, LightMode, MoveState,
    PlayNoteEvent, SelectLayoutEvent, ShiftKeyState, UpdateNoteEvent,
};
use crate::layout::Layout;
use crate::pitch::{Factor, Pitch};
use crate::scale::{Note, ScaleType};
use crate::{controller, csound, events, midi_player};
use anyhow::anyhow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(test)]
mod tests;

mod keys {
    // Top Row, left to right
    pub const SHIFT: u8 = 90;
    pub const MOVE: u8 = 94; // Note
    pub const SUSTAIN: u8 = 95; // Chord
    // Left column, top to bottom
    pub const UP_ARROW: u8 = 80;
    pub const DOWN_ARROW: u8 = 70;
    pub const CLEAR: u8 = 60;
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
    /// control key position -> selected layout
    assigned_layouts: HashMap<u8, Arc<RwLock<Layout>>>,
    transient_state: EngineState,
}

impl Engine {
    async fn reset(&mut self) -> anyhow::Result<()> {
        let config =
            Config::load(&self.config_file).map_err(|e| anyhow!("error reloading config: {e}"))?;
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };

        // Turn off all notes
        for (pitch, count) in &self.transient_state.notes_on {
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
        // TODO: fix these
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
        for (position, label1) in [
            (keys::CLEAR, "Reset"),
            (keys::UP_ARROW, "▲"),
            (keys::DOWN_ARROW, "▼"),
        ] {
            tx.send(Event::Light(LightEvent {
                mode: LightMode::On,
                position,
                color: Color::Active,
                label1: label1.to_string(),
                label2: String::new(),
            }))?;
        }
        let mut position = keys::LAYOUT_MIN;
        self.assigned_layouts.clear();
        for layout in config.layouts.into_iter() {
            tx.send(Event::AssignLayout(AssignLayoutEvent { position, layout }))?;
            position += 1;
            if position > keys::LAYOUT_MAX {
                //TODO: deal with scrolling. Key 109 is the scroll key and will assign layouts
                // starting from an offset to the lower numbered keys.
                tx.send(Event::Light(LightEvent {
                    mode: LightMode::On,
                    position: keys::LAYOUT_SCROLL,
                    color: Color::Active,
                    label1: "Scroll".to_string(),
                    label2: "layouts".to_string(),
                }))?;
                break;
            }
        }
        #[cfg(test)]
        self.send_test_event(TestEvent::ResetComplete);
        Ok(())
    }

    async fn handle_key(&mut self, key_event: KeyEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let KeyEvent { key, velocity } = key_event;
        let off = velocity == 0;
        let have_layout = self.transient_state.layout.is_some();
        if !off && matches!(self.transient_state.shift_key, ShiftKeyState::Down) {
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
                let shift = match (self.transient_state.shift_key, off) {
                    (ShiftKeyState::Off, false) => ShiftKeyState::Down,
                    (ShiftKeyState::Down, true) => ShiftKeyState::On,
                    (ShiftKeyState::On, _) => ShiftKeyState::Off,
                    _ => self.transient_state.shift_key,
                };
                self.set_shift(shift, &tx)?;
            }
            keys::LAYOUT_MIN..=keys::LAYOUT_MAX if off => {
                if let Some(layout) = self.assigned_layouts.get(&key).cloned() {
                    tx.send(Event::SelectLayout(SelectLayoutEvent { layout }))?;
                }
            }
            keys::CLEAR if off => {
                tx.send(Event::Reset)?;
            }
            keys::SUSTAIN if off => {
                self.transient_state.sustain = !self.transient_state.sustain;
                tx.send(self.sustain_light_event())?;
            }
            keys::MOVE if off => {
                self.transient_state.move_state = match self.transient_state.move_state {
                    MoveState::Off => MoveState::Pending,
                    _ => MoveState::Off,
                };
                tx.send(self.move_light_event())?;
            }
            keys::UP_ARROW | keys::DOWN_ARROW if off && self.transient_state.layout.is_some() => {
                // 2025-07-22, rust 1.88: "if let guards" are experimental. When stable, we can
                // use one instead of is_some above and get rid of this unwrap.
                let layout = self.transient_state.layout.take().unwrap();
                {
                    let locked = &mut *layout.write().await;
                    let transposition = if key == keys::UP_ARROW {
                        Pitch::new(vec![Factor::new(2, 1, 1, 1)?])
                    } else {
                        Pitch::new(vec![Factor::new(1, 2, 1, 1)?])
                    };
                    locked.scale.transpose(transposition);
                }
                tx.send(Event::SelectLayout(SelectLayoutEvent { layout }))?;
            }
            position if self.transient_state.notes.contains_key(&position) => {
                if let Some(note) = self.transient_state.notes.get(&position).unwrap() {
                    self.handle_note_key(&tx, note.clone(), position, off)?;
                };
            }
            _ => (),
        }
        Ok(())
    }

    fn handle_note_key(
        &mut self,
        tx: &events::UpgradedSender,
        note: Arc<Note>,
        _position: u8,
        off: bool,
    ) -> anyhow::Result<()> {
        self.handle_note_key_normal(tx, note, off)
    }

    fn handle_note_key_normal(
        &mut self,
        tx: &events::UpgradedSender,
        note: Arc<Note>,
        off: bool,
    ) -> anyhow::Result<()> {
        let pitch = &note.pitch;
        let Some(others) = self.transient_state.note_positions.get(pitch) else {
            // This would indicate a bug in which we assigned something to notes
            // without also assigning its position to note positions or otherwise
            // allowed notes and note_positions to get out of sync.
            return Err(anyhow!("note positions is missing for {pitch}"));
        };
        let note_count = self
            .transient_state
            .notes_on
            .entry(pitch.clone())
            .or_default();
        // When not in sustain mode, touch turns a note on, and release turns it off.
        // Since the same note may appear in multiple locations, we keep a count, and on
        // send a note event if we transition to or from 0. In sustain mode, "off"
        // events are ignored. Touching a note in any of its positions toggles whether
        // it's on or off. Changing scales, transposing, shifting, etc. doesn't affect
        // which notes are on or off, making it possible to play a note in one scale,
        // switch scales, and play a note in another scale.
        if self.transient_state.sustain {
            if off {
                return Ok(());
            }
            if *note_count > 0 {
                *note_count = 0;
            } else {
                *note_count = 1;
            }
        } else if off {
            if *note_count > 0 {
                *note_count -= 1
            }
            if *note_count > 0 {
                return Ok(());
            }
        } else {
            *note_count += 1;
            if *note_count > 1 {
                return Ok(());
            }
        }
        let velocity = if *note_count > 0 { 127 } else { 0 };
        for position in others.iter().copied() {
            tx.send(note.light_event(position, velocity))?;
        }
        tx.send(Event::PlayNote(PlayNoteEvent {
            pitch: pitch.clone(),
            velocity,
            note: Some(note.clone()),
        }))?;
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
                    .note_positions
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

    async fn draw_edo_layout(&mut self, layout: &mut Layout) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let ScaleType::EqualDivision(ed) = &layout.scale.scale_type else {
            // Should not be possible
            return Err(anyhow!("draw_edo_layout called with non-EDO scale"));
        };
        let (llx, lly, urx, ury) = layout.bbox;
        let (steps_x, steps_y) = layout.steps;
        let (base_x, base_y) = layout.base;
        let (divisions, _, _) = ed.divisions;
        let divisions = divisions as i32;
        self.transient_state.note_positions.clear();
        self.transient_state.notes.clear();
        for row in 1..=8 {
            for col in 1..=8 {
                let played_note = if !(llx..=urx).contains(&col) || !(lly..=ury).contains(&row) {
                    None
                } else {
                    let steps = (steps_x * (col - base_x) + steps_y * (row - base_y)) as i32;
                    let cycle = steps / divisions;
                    let step = steps % divisions;
                    let note = layout.scale.note(cycle as i8, step as i8);
                    let velocity = if self
                        .transient_state
                        .notes_on
                        .get(&note.pitch)
                        .copied()
                        .unwrap_or_default()
                        > 0
                    {
                        127
                    } else {
                        0
                    };
                    Some(PlayedNote {
                        note: Arc::new(note),
                        velocity,
                    })
                };
                let position = (10 * row + col) as u8;
                tx.send(Event::UpdateNote(UpdateNoteEvent {
                    position,
                    played_note,
                }))?;
            }
        }
        log::info!("layout: {}, scale: {}", layout.name, layout.scale.name);
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

    fn sustain_light_event(&self) -> Event {
        self.toggle_light_event(self.transient_state.sustain, keys::SUSTAIN, "Sustain", "")
    }

    fn move_light_event(&self) -> Event {
        self.toggle_light_event(
            !matches!(self.transient_state.move_state, MoveState::Off),
            keys::MOVE,
            "Move/",
            "Transpose",
        )
    }

    fn set_shift(
        &mut self,
        shift: ShiftKeyState,
        tx: &events::UpgradedSender,
    ) -> anyhow::Result<()> {
        if shift == self.transient_state.shift_key {
            return Ok(());
        }
        self.transient_state.shift_key = shift;
        tx.send(self.shift_light_event())?;
        Ok(())
    }

    fn shift_light_event(&self) -> Event {
        let color = match self.transient_state.shift_key {
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
        let layout_lock = event.layout;
        self.transient_state.layout = Some(layout_lock.clone());
        let layout = &mut *layout_lock.write().await;
        match layout.scale.scale_type {
            ScaleType::EqualDivision(_) => self.draw_edo_layout(layout).await?,
            ScaleType::_KeepClippyQuiet => unreachable!(),
        }
        tx.send(self.sustain_light_event())?;
        tx.send(self.move_light_event())?;
        tx.send(self.shift_light_event())?;
        #[cfg(test)]
        self.send_test_event(TestEvent::LayoutSelected);
        Ok(())
    }

    async fn assign_layout(&mut self, layout_event: AssignLayoutEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        // Activate the light for selecting the layout.
        let AssignLayoutEvent { position, layout } = layout_event;
        if !(keys::LAYOUT_MIN..=keys::LAYOUT_MAX).contains(&position) {
            return Ok(());
        }
        let (label1, label2) = {
            let l = layout.read().await;
            (l.name.clone(), l.scale.name.clone())
        };
        self.assigned_layouts.insert(position, layout);
        tx.send(Event::Light(LightEvent {
            mode: LightMode::On,
            position,
            color: Color::Active,
            label1,
            label2,
        }))?;
        Ok(())
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
        assigned_layouts: Default::default(),
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
        match event {
            Event::Shutdown => break,
            Event::Light(_) => {}
            Event::Key(e) => engine.handle_key(e).await?,
            Event::Pressure(_) => {}
            Event::Reset => engine.reset().await?,
            Event::AssignLayout(e) => engine.assign_layout(e).await?,
            Event::SelectLayout(e) => engine.select_layout(e).await?,
            Event::UpdateNote(e) => engine.update_note(e).await?,
            Event::PlayNote(_) => {}
            #[cfg(test)]
            Event::TestEngine(test_tx) => test_tx.send(engine.transient_state.clone()).await?,
            #[cfg(test)]
            Event::TestWeb(_) => {}
            #[cfg(test)]
            Event::TestEvent(_) => {}
        }
    }
    Ok(())
}
