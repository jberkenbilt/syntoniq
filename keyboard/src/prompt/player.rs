use crate::events;
use crate::events::{Event, PlayNoteEvent};
use crate::prompt::LineData;
use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Display, Formatter};
use std::mem;
use syntoniq_common::parsing::score;
use syntoniq_common::parsing::score::{DivisionsAndCycle, PromptCommand, ReplNote};
use syntoniq_common::pitch::Pitch;
use tokio::sync::mpsc;

// Keep HELP consistent with actual parser behavior and the prompt-mode.md section of the
// manual.
pub const HELP: &str = r#"** Commands **
?               -- show this help and current state
!!!             -- reset all state except variables
!!              -- silence all notes
>>              -- reset transposition to 1
= pitch         -- set absolute base pitch
* pitch         -- apply relative factor to base pitch
% a             -- set the cycle ratio to `a`
% a/b           -- set the cycle ratio to `a/b`
!               -- use just intonation
!n              -- align with n divisions of the octave
!a/n            -- align with n divisions of `a`
!a/b/n          -- align with n divisions of `a/b`
note1 > note2   -- transpose to give note1's pitch to note2
note > $var     -- save note's pitch into variable $var
$var > note     -- transpose to set note's pitch to the value in $var
note            -- play note, assigning to the lowest available note number
$var            -- show the pitch saved into $var, if any
n < note        -- play note as note n, replacing any existing value
n <             -- stop playing note n
** All notes use generated note syntax. **
Exit with CTRL-C or CTRL-D.
"#;

pub(crate) struct Player {
    tx: events::WeakSender,
    state: PlayerState,
}

struct PlayerState {
    pitches: HashMap<Pitch, usize>,
    notes: BTreeMap<u8, NoteData>,
    divisions_and_cycle: DivisionsAndCycle,
    base_pitch: Pitch,
    transposition: Pitch,
    variables: BTreeMap<String, Pitch>,
}
impl Default for PlayerState {
    fn default() -> Self {
        Self {
            pitches: Default::default(),
            notes: Default::default(),
            divisions_and_cycle: Default::default(),
            base_pitch: Pitch::must_parse("220*^1|4"),
            transposition: Pitch::default(),
            variables: Default::default(),
        }
    }
}

#[derive(Default, Debug, Clone)]
struct NoteData {
    name: String,
    base: Pitch,
    transposition: Pitch,
    relative_with_cycle: Pitch,
    computed: Pitch,
}
impl Display for NoteData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let NoteData {
            name,
            base,
            transposition,
            relative_with_cycle,
            computed,
        } = self;
        let freq = format!("{:.3}", computed.as_float());
        write!(
            f,
            "{name} = {computed} ({base} × {transposition} × {relative_with_cycle}) ≈ {freq}"
        )
    }
}

fn first_gap<T>(map: &BTreeMap<u8, T>) -> Option<u8> {
    let mut gap = 0u8;
    for &k in map.keys() {
        if k != gap {
            return Some(gap);
        }
        gap = gap.checked_add(1)?;
    }
    Some(gap)
}

impl Player {
    pub fn new(events_tx: events::WeakSender) -> Self {
        Self {
            tx: events_tx,
            state: Default::default(),
        }
    }

    pub async fn reset(&mut self) {
        self.clear().await;
        println!("resetting state");
        let variables = mem::take(&mut self.state.variables);
        self.state = Default::default();
        self.state.variables = variables;
    }

    pub async fn clear(&mut self) {
        println!("turning off all notes");
        self.state.notes.clear();
        for pitch in mem::take(&mut self.state.pitches).into_keys() {
            self.do_play(pitch, false).await;
        }
    }

    pub async fn handle_lines(
        &mut self,
        mut line_rx: mpsc::Receiver<LineData>,
    ) -> anyhow::Result<()> {
        while let Some(line) = line_rx.recv().await {
            if line.line.is_empty() {
                continue;
            }
            if line.line.trim() == "?" {
                print!("{HELP}");
                println!("base pitch = {}", self.state.base_pitch);
                println!("transposition = {}", self.state.transposition);
                println!("divisions = {}", self.state.divisions_and_cycle.divisions);
                println!("cycle ratio = {}", self.state.divisions_and_cycle.cycle);
                if !self.state.variables.is_empty() {
                    println!("variables:");
                    for (v, p) in &self.state.variables {
                        println!("  {v} = {p}");
                    }
                }
                if !self.state.notes.is_empty() {
                    println!("current notes:");
                    for (&k, v) in &self.state.notes {
                        println!("  {k:-3}: {v}");
                    }
                }
                continue;
            }
            let Some(command) =
                score::parse_prompt_line(&line.line, &self.state.divisions_and_cycle)
            else {
                continue;
            };
            self.handle_command(command).await;
        }
        Ok(())
    }

    async fn do_play(&mut self, pitch: Pitch, on: bool) {
        let Some(tx) = self.tx.upgrade() else {
            return;
        };
        let mut velocity: Option<u8> = None;
        // Adjust pitch count and decide whether to take action.
        match self.state.pitches.entry(pitch.clone()) {
            Entry::Occupied(mut e) => {
                let count = e.get_mut();
                if on {
                    // Increment the count of already-playing pitch
                    *count += 1;
                } else {
                    *count -= 1;
                    if *count == 0 {
                        e.remove();
                        // This is the last occurrence of the pitch, so turn it off.
                        velocity = Some(0);
                    }
                }
            }
            Entry::Vacant(e) => {
                if on {
                    // This is the first occurrence of the pitch, so play it.
                    e.insert(1);
                    velocity = Some(127);
                } else {
                    // Clear clears self.pitches, so treat this as an unconditional stop.
                    velocity = Some(0);
                }
            }
        };
        if let Some(velocity) = velocity {
            _ = tx.send(Event::PlayNote(PlayNoteEvent {
                pitch,
                velocity,
                note: None,
            }));
        }
    }

    async fn handle_play(&mut self, n: Option<u8>, note: Option<ReplNote>) {
        if let Some(i) = n
            && let Some(old) = self.state.notes.remove(&i)
        {
            println!("-     {old}");
            self.do_play(old.computed, false).await;
        }
        let mut added: Option<u8> = None;
        if let Some(note) = note {
            let Some(i) = n.or_else(|| first_gap(&self.state.notes)) else {
                // No pop culture reference intended.
                println!("too many notes");
                return;
            };
            let computed = note
                .pitch
                .clone()
                .concat(&self.state.base_pitch)
                .concat(&self.state.transposition);
            let note_data = NoteData {
                name: note.name,
                base: self.state.base_pitch.clone(),
                transposition: self.state.transposition.clone(),
                relative_with_cycle: note.pitch.clone(),
                computed: computed.clone(),
            };
            self.do_play(computed, true).await;
            self.state.notes.insert(i, note_data);
            added = Some(i)
        }
        for (&k, v) in &self.state.notes {
            let mark = if let Some(i) = added
                && k == i
            {
                '*'
            } else {
                ' '
            };
            println!("{mark}{k:-3}: {v}");
        }
    }

    async fn handle_command(&mut self, command: PromptCommand) {
        match command {
            PromptCommand::Reset => self.reset().await,
            PromptCommand::Clear => self.clear().await,
            PromptCommand::ResetTransposition => {
                self.state.transposition = Pitch::unit();
                println!("transposition = 1");
            }
            PromptCommand::SetDivisions { divisions } => {
                self.state.divisions_and_cycle.divisions = divisions;
                println!("divisions = {}", self.state.divisions_and_cycle.divisions);
            }
            PromptCommand::SetCycleRatio { cycle } => {
                self.state.divisions_and_cycle.cycle = cycle;
                println!("cycle ratio = {}", self.state.divisions_and_cycle.cycle);
            }
            PromptCommand::SetBaseAbsolute { pitch } => {
                self.state.base_pitch = pitch;
                println!("base pitch = {}", self.state.base_pitch);
            }
            PromptCommand::SetBaseRelative { pitch } => {
                self.state.base_pitch *= &pitch;
                println!("base pitch = {}", self.state.base_pitch);
            }
            PromptCommand::Transpose {
                pitch_from,
                written,
            } => {
                let p = &pitch_from.pitch / &written.pitch;
                self.state.transposition *= &p;
                println!("transposition = {}", self.state.transposition);
            }
            PromptCommand::Play { n, note } => self.handle_play(n, note).await,
            PromptCommand::Save { note, variable } => {
                self.state
                    .variables
                    .insert(variable, &note.pitch * &self.state.transposition);
            }
            PromptCommand::Restore { note, variable } => {
                match self.state.variables.get(&variable) {
                    Some(pitch) => {
                        println!("{variable} = {pitch}");
                        self.state.transposition = pitch / &note.pitch;
                        println!("transposition = {}", self.state.transposition);
                    }
                    None => println!("unknown variable {variable}"),
                }
            }
            PromptCommand::ShowVar { variable } => match self.state.variables.get(&variable) {
                Some(pitch) => println!("{variable} = {pitch}"),
                None => println!("unknown variable {variable}"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::Events;

    #[tokio::test]
    async fn test_player() {
        let events = Events::new();
        let events_tx = events.sender().await;
        let _events_rx = events.receiver();
        let mut p = Player::new(events_tx);

        async fn cmd(p: &mut Player, s: &str) {
            p.handle_command(score::parse_prompt_line(s, &p.state.divisions_and_cycle).unwrap())
                .await;
        }

        cmd(&mut p, "A").await;
        assert!(p.state.notes.contains_key(&0));
        assert_eq!(
            *p.state.pitches.get(&Pitch::must_parse("220*^1|4")).unwrap(),
            1
        );
        cmd(&mut p, "A").await;
        assert_eq!(
            *p.state.pitches.get(&Pitch::must_parse("220*^1|4")).unwrap(),
            2
        );
        cmd(&mut p, "0<").await;
        assert_eq!(
            *p.state.pitches.get(&Pitch::must_parse("220*^1|4")).unwrap(),
            1
        );
        cmd(&mut p, "1<").await;
        assert!(!p.state.pitches.contains_key(&Pitch::must_parse("220*^1|4")));
        cmd(&mut p, "I > A").await;
        cmd(&mut p, "= 500").await;
        cmd(&mut p, "* 6/5").await;
        cmd(&mut p, "C").await;
        assert_eq!(
            *p.state
                .pitches
                .get(&Pitch::must_parse("600*3/2*9/8"))
                .unwrap(),
            1
        );
        assert_eq!(p.state.pitches.len(), 1);
        assert_eq!(p.state.notes.len(), 1);
        cmd(&mut p, "!!").await;
        assert!(p.state.pitches.is_empty());
        assert!(p.state.notes.is_empty());
        assert_eq!(p.state.base_pitch, Pitch::must_parse("600"));
        assert_eq!(p.state.transposition, Pitch::must_parse("9/8"));
        cmd(&mut p, "!!!").await;
        assert_eq!(p.state.base_pitch, Pitch::must_parse("220*^1|4"));
        assert_eq!(p.state.transposition, Pitch::unit());
        cmd(&mut p, "!3/27").await;
        cmd(&mut p, "C'").await;
        assert!(
            p.state
                .pitches
                .contains_key(&Pitch::must_parse("2*220*^1|4*3^10|27"))
        );
        cmd(&mut p, "%5").await;
        cmd(&mut p, "C,").await;
        assert!(
            p.state
                .pitches
                .contains_key(&Pitch::must_parse("1/5*220*^1|4*3^10|27"))
        );
        cmd(&mut p, "!!!").await;
        cmd(&mut p, "!67").await;
        cmd(&mut p, "M>A").await;
        cmd(&mut p, "A>$A").await;
        assert_eq!(
            p.state.variables.get("$A").unwrap(),
            &Pitch::must_parse("^8|67")
        );
        assert_eq!(p.state.transposition, Pitch::must_parse("^8|67"));
        cmd(&mut p, "G>A").await;
        assert_eq!(p.state.transposition, Pitch::must_parse("^23|67"));
        cmd(&mut p, "$A>A").await;
        assert_eq!(p.state.transposition, Pitch::must_parse("^8|67"));
        events.shutdown().await;
    }
}
