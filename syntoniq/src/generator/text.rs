use std::fs::File;
use std::io::Write;
use std::path::Path;
use syntoniq_common::parsing::{PitchChange, Timeline, TimelineData, TimelineEvent};

pub(crate) fn generate(timeline: &Timeline, out: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut f = File::create(out)?;
    for event in &timeline.events {
        let TimelineEvent {
            time,
            repeat_depth,
            span,
            data,
        } = event.as_ref();
        let indent = " ".repeat(*repeat_depth);
        write!(f, "{indent} {time}: ")?;
        match data {
            TimelineData::Tempo(e) => {
                write!(f, "tempo: {}", e.bpm)?;
                if let Some(end_bpm) = &e.end_bpm {
                    write!(
                        f,
                        " .. {end} at {time}",
                        end = end_bpm.item,
                        time = end_bpm.time
                    )?;
                }
            }
            TimelineData::Dynamic(e) => {
                write!(f, "[{part}] @{level}", part = e.part, level = e.start_level)?;
                if let Some(end_level) = &e.end_level {
                    write!(
                        f,
                        " .. {end} at {time}",
                        end = end_level.item,
                        time = end_level.time
                    )?;
                }
            }
            TimelineData::Note(e) => {
                write!(
                    f,
                    "[{part}.{note}] v={vel}",
                    part = e.part_note.part,
                    note = e.part_note.note_number,
                    vel = e.value.velocity
                )?;
                let mut first = true;
                for p in &e.value.pitches {
                    write!(f, "\n{indent}   ")?;
                    if first {
                        write!(f, "   ")?;
                        first = false;
                    } else {
                        write!(f, "-> ")?;
                    }
                    let PitchChange {
                        text,
                        span,
                        start_pitch,
                        start_time,
                        end_pitch,
                        end_time,
                    } = p;
                    write!(f, "{text} = {start_pitch} at {start_time}")?;
                    if let Some(end_pitch) = end_pitch.as_ref() {
                        write!(f, " .. {end_pitch}")?;
                    }
                    write!(f, " until {end_time} {span}")?;
                }
            }
            TimelineData::Mark(e) => {
                write!(f, "mark {}", e.label)?;
            }
            TimelineData::RepeatStart(e) => {
                write!(f, "begin repeat from {}", e.label)?;
            }
            TimelineData::RepeatEnd(e) => {
                write!(f, "end repeat at {}", e.label)?;
            }
        }
        if !matches!(data, TimelineData::Note(_)) {
            write!(f, " {span}")?;
        }
        writeln!(f)?;
    }
    Ok(())
}
