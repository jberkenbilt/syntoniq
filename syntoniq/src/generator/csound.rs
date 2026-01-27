use anyhow::{anyhow, bail};
use num_rational::Ratio;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::{cmp, fs};
use syntoniq_common::parsing::{CsoundInstrumentId, Timeline, TimelineData};

pub(crate) const DEFAULT_TEMPLATE: &str = include_str!("csound-template.csd");

struct PartData {
    part_number: usize,
    note_numbers: BTreeMap<u32, usize>,
}

struct CsoundGenerator<'s> {
    timeline: &'s Timeline<'s>,
    content: String,
    part_data: BTreeMap<&'s str, PartData>,
}

pub fn rounded_float(val: impl Into<f64>, max_decimals: usize) -> String {
    let mut s = format!("{:.*}", max_decimals, val.into());
    if s.contains('.') {
        s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    s
}

pub fn ratio_to_rounded_float(r: Ratio<u32>, max_decimals: usize) -> String {
    let val = (*r.numer() as f64) / (*r.denom() as f64);
    rounded_float(val, max_decimals)
}

pub fn scale_dynamic(v: u8, non_zero: bool) -> String {
    let v = if non_zero && v == 0 { 1 } else { v };
    rounded_float(v as f32 / 127.0, 3)
}

fn pad_number(n: impl Into<usize>, max_n: usize) -> String {
    let width = max_n.to_string().len();
    format!("{:0width$}", n.into(), width = width)
}

impl<'s> CsoundGenerator<'s> {
    fn new(timeline: &'s Timeline) -> anyhow::Result<Self> {
        Ok(Self {
            timeline,
            content: Default::default(),
            part_data: Default::default(),
        })
    }

    fn instrument_for_part(&self, part: &str) -> CsoundInstrumentId<'s> {
        self.timeline
            .csound_instruments
            .get(part)
            .or_else(|| self.timeline.csound_instruments.get(""))
            .cloned()
            .unwrap_or(CsoundInstrumentId::Number(1))
    }

    fn assign_part_numbers(&mut self) -> anyhow::Result<()> {
        // Assign a number to each part. Number from 1.
        let mut next_part = 1;
        let mut next_note_number_by_instrument: HashMap<CsoundInstrumentId, usize> = HashMap::new();
        let mut max_notes_by_instrument: HashMap<CsoundInstrumentId, usize> = HashMap::new();
        let mut cur_notes_by_instrument: HashMap<CsoundInstrumentId, HashSet<u32>> = HashMap::new();
        for event in &self.timeline.events {
            let TimelineData::Note(e) = &event.data else {
                continue;
            };
            let instrument = self.instrument_for_part(e.part_note.part);
            // Track notes by instrument for polyphonic amplitude adjustment.
            let notes_on = cur_notes_by_instrument
                .entry(instrument.clone())
                .or_default();
            if e.value.velocity == 0 {
                notes_on.remove(&e.part_note.note_number);
            } else if notes_on.insert(e.part_note.note_number) {
                let note_count = notes_on.len();
                let max_notes = max_notes_by_instrument
                    .entry(instrument.clone())
                    .or_default();
                *max_notes = cmp::max(*max_notes, note_count);
            }

            let part_data = self.part_data.entry(e.part_note.part).or_insert_with(|| {
                let new_value = PartData {
                    part_number: next_part,
                    note_numbers: Default::default(),
                };
                next_part += 1;
                new_value
            });

            if let Entry::Vacant(nv) = part_data.note_numbers.entry(e.part_note.note_number) {
                // Number notes from 1
                let note_number = next_note_number_by_instrument
                    .entry(instrument)
                    .or_insert(1);
                nv.insert(*note_number);
                *note_number += 1;
            }
        }
        self.content.push_str("; [part] => csound part\n");
        for (part, data) in &self.part_data {
            let pn = data.part_number;
            self.content.push_str(&format!("; [{part}] => {pn}\n"));
        }
        self.content.push_str("; [part.note] => instr.note\n");
        for (part, data) in &self.part_data {
            let instr = self.instrument_for_part(part).output(None);
            for (stq, csd) in &data.note_numbers {
                self.content
                    .push_str(&format!("; [{part}.{stq}] => {instr}.{csd}\n"));
            }
        }
        self.content.push('\n');
        for (&part_str, data) in &self.part_data {
            let instrument = self.instrument_for_part(part_str);
            let max_notes = max_notes_by_instrument
                .get(&instrument)
                .copied()
                .unwrap_or(1);
            self.content.push_str(&format!(
                "i \"SetPartParam\" 0 0.01 {} \"amp\" 0.5\n",
                data.part_number
            ));
            self.content.push_str(&format!(
                "i \"SetPartParam\" 0 0.01 {} \"notes\" {max_notes}\n",
                data.part_number
            ));
        }
        Ok(())
    }

    fn compute_tempo(&mut self) -> anyhow::Result<()> {
        let mut points = Vec::new();
        for event in &self.timeline.events {
            let TimelineData::Tempo(e) = &event.data else {
                continue;
            };
            let time = ratio_to_rounded_float(event.time, 3);
            if let Some(prev) = points.last().cloned() {
                // Repeat the previous tempo at the current time to effect an instantaneous tempo
                // change
                points.push(time.clone());
                points.push(prev.clone());
            }
            // Set current tempo
            let bpm = ratio_to_rounded_float(e.bpm, 3);
            points.push(time);
            points.push(bpm);
            if let Some(v) = &e.end_bpm {
                let time = ratio_to_rounded_float(v.time, 3);
                let bpm = ratio_to_rounded_float(v.item, 3);
                points.push(time);
                points.push(bpm);
            }
        }
        if points.is_empty() {
            self.content.push_str("t 0 72\n");
        } else {
            self.content.push_str(&format!("t {}\n", points.join(" ")));
        }
        Ok(())
    }

    fn analyze(&mut self) -> anyhow::Result<()> {
        self.assign_part_numbers()?;
        self.compute_tempo()?;
        Ok(())
    }

    fn generate(mut self) -> anyhow::Result<String> {
        self.analyze()?;
        for event in &self.timeline.events {
            let time = ratio_to_rounded_float(event.time, 3);
            let offset = event.span.start;
            match &event.data {
                TimelineData::Tempo(_) => { /* already handled */ }
                TimelineData::Dynamic(e) => {
                    let &part_data = &self
                        .part_data
                        .get(e.part)
                        .ok_or_else(|| anyhow!("unknown part"))?;
                    let part_number = part_data.part_number;
                    let comment = format!("; {} @{offset}", e.text);
                    match &e.end_level {
                        None => {
                            let start = scale_dynamic(e.start_level, false);
                            self.content.push_str(&format!(
                                "i \"SetPartParam\" {time} 0.01 {part_number} \"amp\" {start} {comment}\n",
                            ));
                        }
                        Some(end_level) => {
                            // Scale dynamics to 0..1 excluding 0 since cSound's `expseg` doesn't
                            // support 0.
                            let start = scale_dynamic(e.start_level, true);
                            let end = scale_dynamic(end_level.item, true);
                            let duration = ratio_to_rounded_float(end_level.time - event.time, 3);
                            self.content.push_str(
                                &format!(
                                    "i \"SetPartParamRamp\" {time} {duration} {part_number} \"amp\" {start} {end} {comment}\n",
                                ),
                            );
                            if end_level.item == 0 {
                                // Generate a zero-volume event at the end if we're going to silence.
                                let time = ratio_to_rounded_float(end_level.time, 3);
                                self.content.push_str(&format!(
                                    "i \"SetPartParam\" {time} 0.01 {part_number} \"amp\" 0\n",
                                ));
                            }
                        }
                    }
                }
                TimelineData::Note(e) => {
                    let instrument = self.instrument_for_part(e.part_note.part);
                    let &part_data = &self
                        .part_data
                        .get(e.part_note.part)
                        .ok_or_else(|| anyhow!("unknown part"))?;
                    let part_number = part_data.part_number;
                    let note_number = part_data
                        .note_numbers
                        .get(&e.part_note.note_number)
                        .ok_or_else(|| anyhow!("unknown note number"))?;
                    let velocity = ratio_to_rounded_float(
                        Ratio::new(cmp::min(127, e.value.velocity as u32), 127),
                        3,
                    );
                    // instrument.note is a decimal number, so we need to use leading zeroes based
                    // on the number of note numbers.
                    let instr_note_number = pad_number(*note_number, part_data.note_numbers.len());
                    let instr = instrument.output(Some(instr_note_number));
                    for pc in &e.value.pitches {
                        let start_time = ratio_to_rounded_float(pc.start_time, 3);
                        let duration = ratio_to_rounded_float(pc.end_time - pc.start_time, 3);
                        let freq = rounded_float(pc.start_pitch.as_float(), 3);
                        let text = pc.text;
                        let this_offset = pc.span.start;
                        self.content.push_str(&format!("; {text}@{this_offset}\n"));
                        match pc.end_pitch.as_ref() {
                            None => {
                                self.content.push_str(&format!(
                                    "i \"SetPartParam\" {start_time} {duration} {part_number} \"freq_{note_number}\" {freq}\n"
                                ));
                            }
                            Some(end_pitch) => {
                                let end_freq = rounded_float(end_pitch.as_float(), 3);
                                self.content.push_str(&format!(
                                    "i \"SetPartParamRamp\" {start_time} {duration} {part_number} \"freq_{note_number}\" {freq} {end_freq}\n"
                                ));
                            }
                        }
                    }
                    let note_text = e.value.text;
                    let end_time = e.value.pitches.last().unwrap().end_time;
                    if event.time > end_time {
                        bail!("end time is in the past")
                    }
                    let duration = ratio_to_rounded_float(end_time - event.time, 3);
                    self.content.push_str(&format!("; {note_text} @{offset}\n"));
                    self.content.push_str(&format!(
                        "i {instr} {time} {duration} {part_number} {note_number} {velocity}\n"
                    ));
                }
                TimelineData::Mark(e) => {
                    self.content
                        .push_str(&format!("; mark '{}' @'{}\n", e.label, event.span));
                }
                TimelineData::RepeatStart(e) => {
                    self.content
                        .push_str(&format!("; repeat start '{}' @'{}\n", e.label, event.span));
                }
                TimelineData::RepeatEnd(e) => {
                    self.content
                        .push_str(&format!("; repeat end '{}' @'{}\n", e.label, event.span));
                }
            }
        }
        Ok(self.content)
    }
}

pub(crate) fn generate(
    timeline: &Timeline,
    out: impl AsRef<Path>,
    template_override: Option<impl AsRef<Path>>,
) -> anyhow::Result<()> {
    let g = CsoundGenerator::new(timeline)?;
    let content = g.generate()?;
    let mut _loaded: Option<String> = None;
    let template = match template_override {
        Some(path) => {
            _loaded = Some(String::from_utf8(fs::read(path)?)?);
            _loaded.as_ref().unwrap().as_str()
        }
        None => DEFAULT_TEMPLATE,
    };
    const BEGIN_MARK: &str = ";; BEGIN SYNTONIQ";
    const END_MARK: &str = ";; END SYNTONIQ";
    let begin_pos = template
        .find(BEGIN_MARK)
        .ok_or_else(|| anyhow!("csound template doesn't contain '{BEGIN_MARK}'"))?;
    let end_pos = template
        .find(END_MARK)
        .ok_or_else(|| anyhow!("csound template doesn't contain '{END_MARK}'"))?;
    fs::write(
        &out,
        format!(
            "{}{BEGIN_MARK}\n{content}{}",
            &template[..begin_pos],
            &template[end_pos..]
        ),
    )?;
    println!("Csound output written to {}", out.as_ref().display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trunc_examples() {
        assert_eq!(ratio_to_rounded_float(Ratio::new(1, 3), 4), "0.3333");
        assert_eq!(ratio_to_rounded_float(Ratio::new(10, 4), 2), "2.5");
        assert_eq!(ratio_to_rounded_float(Ratio::new(7, 2), 0), "4");
        assert_eq!(
            ratio_to_rounded_float(Ratio::new(999_999_999, 1), 3),
            "999999999"
        );
    }

    #[test]
    fn test_pad_number() {
        assert_eq!(pad_number(12u8, 100), "012");
        assert_eq!(pad_number(1256u16, 100), "1256");
        assert_eq!(pad_number(3usize, 10), "03");
    }
}
