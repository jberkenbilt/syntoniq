// -*- fill-column: 80 -*-
use crate::parsing::diagnostics::{Diagnostics, code};
use crate::parsing::model::{Span, Spanned};
use crate::parsing::score::HashSet;
use crate::parsing::score::RawDirective;
use crate::parsing::score_helpers;
use crate::pitch::Pitch;
use directive_derive::FromRawDirective;
use num_rational::Ratio;
use std::io;

pub trait FromRawDirective: Sized {
    fn from_raw(diags: &Diagnostics, d: &RawDirective) -> Option<Self>;
    fn show_help(w: &mut impl io::Write) -> io::Result<()>;
}

#[derive(FromRawDirective)]
/// Set the syntoniq file format version. This must be the first functional item
/// in the file.
pub struct Syntoniq {
    pub span: Span,
    /// syntoniq file format version; supported value: 1
    pub version: Spanned<u32>,
}
impl Syntoniq {
    pub fn validate(&mut self, diags: &Diagnostics) {
        if self.version.value != 1 {
            diags.err(
                code::DIRECTIVE_USAGE,
                self.version.span,
                "syntoniq version must be 1",
            );
        }
    }
}

#[derive(FromRawDirective)]
/// Define a scale. The scale called "default" is pre-defined and corresponds to
/// 12-EDO.
pub struct DefineScale {
    pub span: Span,
    /// scale name
    pub name: Spanned<String>,
    /// ratio to be applied by the octave marker; default is 2 (one octave)
    pub cycle_ratio: Option<Spanned<Ratio<u32>>>,
}
impl DefineScale {
    pub fn validate(&mut self, _diags: &Diagnostics) {}
}

#[derive(FromRawDirective)]
/// Change the scale for the specified parts. If no parts are specified, change the scale used
/// by parts with no explicit scale. This creates a tuning with the specified scale and the current
/// base pitch.
pub struct UseScale {
    pub span: Span,
    /// Scale name
    pub name: Spanned<String>,
    /// Which parts the tune; if not specified, all parts are tuned
    pub part: Vec<Spanned<String>>,
}
impl UseScale {
    pub fn validate(&mut self, diags: &Diagnostics) {
        score_helpers::check_part(diags, &self.part);
    }
}

#[derive(FromRawDirective)]
/// Change the base pitch of the scale in a way that makes the new pitch of `written` equal to the
/// current pitch of `pitch_from`. For example, you could transpose up a whole step in 12-TET with
/// `transpose(written="c" pitch_from="d")`. This method of specifying transposition is easily
/// reversible even in non-EDO tunings by simply swapping `written` and `pitch_from`. This can be
/// applied to multiple parts or to the default tuning. The parts do not all have to be using the
/// same scale as long as they are all using scales that have both named notes.
pub struct Transpose {
    pub span: Span,
    /// Name of note used as anchor pitch for transposition. In the new tuning, this note will have
    /// the pitch that the note in `pitch_from` has before the transposition.
    pub written: Spanned<String>,
    /// Name of the note in the existing tuning whose pitch will be given to the `written` note
    /// after transposition.
    pub pitch_from: Spanned<String>,
    /// Which parts the tune; if not specified, all parts are tuned
    pub part: Vec<Spanned<String>>,
}
impl Transpose {
    pub fn validate(&mut self, diags: &Diagnostics) {
        score_helpers::check_part(diags, &self.part);
    }
}

#[derive(FromRawDirective)]
/// Change the base pitch of the current tuning for the named parts, or if no parts are named, for
/// the default tuning. If `absolute`, use the pitch as the absolute base pitch. If `relative`,
/// multiply the base pitch by the given factor. Example: `set_base_pitch(relative="^1|12")` would
/// transpose the tuning up one 12-TET half step. Only one of `absolute` or `relative` may be
/// given.
pub struct SetBasePitch {
    pub span: Span,
    /// Set the base pitch of the current tuning to this absolute pitch value
    pub absolute: Option<Spanned<Pitch>>,
    /// Multiply the base pitch of the current tuning by the specified factor
    pub relative: Option<Spanned<Pitch>>,
    /// Which parts the tune; if not specified, all parts are tuned
    pub part: Vec<Spanned<String>>,
}
impl SetBasePitch {
    pub fn validate(&mut self, diags: &Diagnostics) {
        score_helpers::check_part(diags, &self.part);
        let n = [self.absolute.is_some(), self.relative.is_some()]
            .into_iter()
            .fold(0usize, |x, v| x + if v { 1 } else { 0 });
        if n != 1 {
            diags.err(
                code::TUNE,
                self.span,
                "exactly one of absolute or relative must be specified",
            );
        }
    }
}

#[derive(FromRawDirective)]
/// Reset tuning. If no parts are specified, clears all tunings. Otherwise,
/// resets the tuning for each specified part to use the global tuning.
pub struct ResetTuning {
    pub span: Span,
    /// Which parts the tune; if not specified, all parts are tuned
    pub part: Vec<Spanned<String>>,
}
impl ResetTuning {
    pub fn validate(&mut self, diags: &Diagnostics) {
        score_helpers::check_part(diags, &self.part);
    }
}

#[derive(FromRawDirective)]
/// Set the MIDI instrument number for zero or more parts. If no part is specified, this becomes
/// the default instrument for all parts without a specific instrument. It is an error to name
/// a part that doesn't appear somewhere in the score.
pub struct MidiInstrument {
    pub span: Span,
    /// Midi instrument number from 1 to 128
    pub instrument: Spanned<u32>,
    /// Optional bank number from 1 to 16384
    pub bank: Option<Spanned<u32>>,
    /// Which parts use this instrument; if not specified, all unassigned parts use it
    pub part: Vec<Spanned<String>>,
}
impl MidiInstrument {
    pub fn validate(&mut self, diags: &Diagnostics) {
        score_helpers::check_part(diags, &self.part);
        // User-facing numbers are 1-based. We will store as 0-based internally.
        if !(1..=128).contains(&self.instrument.value) {
            diags.err(
                code::MIDI,
                self.instrument.span,
                "instrument numbers must be between 1 and 128",
            );
        }
        if let Some(bank) = self.bank
            && !(1..=16384).contains(&bank.value)
        {
            diags.err(
                code::MIDI,
                bank.span,
                "bank numbers must be between 1 and 16384",
            );
        }
    }
}

#[derive(FromRawDirective)]
/// Set tempo, with possible accelerando or rallentando (gradual change).
pub struct Tempo {
    pub span: Span,
    /// Tempo in beats per minute
    pub bpm: Spanned<Ratio<u32>>,
    /// Optional effective time relative to the current score time. This can be useful
    /// for inserting a tempo change part way through a score line. Defaults to 0.
    pub start_time: Option<Spanned<Ratio<u32>>>,
    /// Optional end tempo; if specified, duration is required. Indicates that the tempo should
    /// change gradually from `bpm` to `end_bpm` over `duration` beats.
    pub end_bpm: Option<Spanned<Ratio<u32>>>,
    /// Must appear with `end_bpm` to indicate the duration of a gradual tempo change.
    pub duration: Option<Spanned<Ratio<u32>>>,
}
impl Tempo {
    pub fn validate(&mut self, diags: &Diagnostics) {
        let n = [self.end_bpm.is_some(), self.duration.is_some()]
            .into_iter()
            .fold(0usize, |x, v| x + if v { 1 } else { 0 });
        if n == 1 {
            diags.err(
                code::USAGE,
                self.span,
                "'end_bpm' and 'duration' must either both be present or both be absent",
            );
        }
    }
}

#[derive(FromRawDirective)]
pub enum Directive {
    Syntoniq(Syntoniq),
    DefineScale(DefineScale),
    UseScale(UseScale),
    Transpose(Transpose),
    SetBasePitch(SetBasePitch),
    ResetTuning(ResetTuning),
    MidiInstrument(MidiInstrument),
    Tempo(Tempo),
}
