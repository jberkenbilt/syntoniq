// -*- fill-column: 80 -*-
use crate::parsing::diagnostics::{Diagnostic, Diagnostics, code};
use crate::parsing::model::{DataBlock, LayoutBlock, ScaleBlock, Span, Spanned};
use crate::parsing::score::HashSet;
use crate::parsing::score::RawDirective;
use crate::parsing::score_helpers;
use crate::pitch::Pitch;
use directive_derive::FromRawDirective;
use num_rational::Ratio;
use std::borrow::Cow;
use std::io;

pub trait FromRawDirective<'s>: Sized {
    fn from_raw(diags: &Diagnostics, span: Span, d: &RawDirective<'s>) -> Option<Self>;
    fn show_help(w: &mut impl io::Write) -> io::Result<()>;
}

#[derive(FromRawDirective)]
/// Set the syntoniq file format version. This must be the first functional item
/// in the file.
pub struct Syntoniq<'s> {
    pub _s: &'s (),
    pub span: Span,
    /// syntoniq file format version; supported value: 1
    pub version: Spanned<u32>,
}
impl<'s> Syntoniq<'s> {
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
pub struct DefineScale<'s> {
    pub span: Span,
    /// scale name
    pub scale: Spanned<Cow<'s, str>>,
    /// ratio to be applied by the octave marker; default is 2 (one octave)
    pub cycle_ratio: Option<Spanned<Ratio<u32>>>,
    pub scale_block: Spanned<ScaleBlock<'s>>,
}
impl<'s> DefineScale<'s> {
    pub fn validate(&mut self, _diags: &Diagnostics) {}
}

#[derive(FromRawDirective)]
/// Define a generated scale. Note pitches are generated according to the
/// following rules:
/// - Notes consist of letters, numbers, +, -, #, and %
/// - `A` and `a` represent the root of the scale
/// - `B` through `Y` represent n/n-1 where n is the ordinal position of the
///   letter (B=2, C=3/2, D=4/3, etc.)
/// - `b` through `y` are n-1/n, the reciprocal of their upper-case
///   counterparts (b=1/2, c=2/3, d=3/4, etc.)
/// - `Z` followed by a number ≥ 2 represents n/n-1 (e.g. Z30 = 30/29)
/// - `z` followed by a number ≥ 2 represents n-n/n (e.g. z30 = 29/30)
/// - All factors are multiplied to create the base pitch; e.g, (Bh = 2*7/8 =
///   7/4, Cl = 3/2*11/12 = 11/8)
///
/// When `divisions` is specified, the following additional rules apply:
/// - `An` represents `n` scale steps up (cycle^n|divisions)
/// - `an` represents `n` scale steps down (cycle^-n|divisions)
/// - `+` is short for `A1` (raises the pitch by one scale degree)
/// - `-` is short for `a1` (lowers the pitch by one scale degree)
/// - If `tolerance` is not specified or the pitch is within tolerance of its
///   nearest scale degree, the pitch is rounded to the nearest scale degree,
///   and the `#` and `%` characters have no effect on the pitch.
/// - If `tolerance` is specified and the pitch is farther away from its nearest
///   scale degree than `tolerance`:
///   - `#` forces the pitch to the next highest scale degree
///   - `%` forces the pitch to the next lowest scale degree
///
/// Example: with divisions = 17 and tolerance of 4¢:
/// - `E` is `^5|17` because 5/4 is between steps 5 and 6 (zero-based) but is
///   slightly closer to step 5
/// - `E%` is also `^5|17`
/// - `E#` is `^6|17`
///
/// See the manual for more details and examples.
pub struct DefineGeneratedScale<'s> {
    pub span: Span,
    /// scale name
    pub scale: Spanned<Cow<'s, str>>,
    /// ratio to be applied by the octave marker; default is 2 (one octave)
    pub cycle_ratio: Option<Spanned<Ratio<u32>>>,
    /// cycle divisions -- omit for a pure Just-Intonation scale
    pub divisions: Option<Spanned<u32>>,
    /// tolerance for `#` and `%` -- `#` and `%` are ignored if computed pitch
    /// is within `tolerance` of a scale degree; allowed only when `divisions`
    /// is given
    pub tolerance: Option<Spanned<Pitch>>,
}
impl<'s> DefineGeneratedScale<'s> {
    pub fn validate(&mut self, diags: &Diagnostics) {
        if let Some(divisions) = self.divisions
            && divisions.value < 2
        {
            diags.err(
                code::DIRECTIVE_USAGE,
                divisions.span,
                "divisions, if specified, must be >= 2",
            );
        }
        if let Some(tolerance) = &self.tolerance
            && self.divisions.is_none()
        {
            diags.err(
                code::DIRECTIVE_USAGE,
                tolerance.span,
                "tolerance is only allowed when divisions is specified",
            )
        }
    }
}

#[derive(FromRawDirective)]
/// Change the scale for the specified parts. If no parts are specified, change
/// the scale used by parts with no explicit scale. This creates a tuning with
/// the specified scale and the current base pitch.
pub struct UseScale<'s> {
    pub span: Span,
    /// Scale name
    pub scale: Spanned<Cow<'s, str>>,
    /// Which parts the tune; if not specified, all parts are tuned
    pub part: Vec<Spanned<Cow<'s, str>>>,
}
impl<'s> UseScale<'s> {
    pub fn validate(&mut self, diags: &Diagnostics) {
        score_helpers::check_part(diags, &self.part);
    }
}

#[derive(FromRawDirective)]
/// Change the base pitch of the scale in a way that makes the new pitch of
/// `written` equal to the current pitch of `pitch_from`. For example, you could
/// transpose up a whole step in 12-TET with `transpose(written="c"
/// pitch_from="d")`. This method of specifying transposition is easily
/// reversible even in non-EDO tunings by simply swapping `written` and
/// `pitch_from`. This can be applied to multiple parts or to the default
/// tuning. The parts do not all have to be using the same scale as long as they
/// are all using scales that have both named notes.
pub struct Transpose<'s> {
    pub span: Span,
    /// Name of note used as anchor pitch for transposition. In the new tuning,
    /// this note will have the pitch that the note in `pitch_from` has before
    /// the transposition.
    pub written: Spanned<Cow<'s, str>>,
    /// Name of the note in the existing tuning whose pitch will be given to the
    /// `written` note after transposition.
    pub pitch_from: Spanned<Cow<'s, str>>,
    /// Which parts the tune; if not specified, all parts are tuned
    pub part: Vec<Spanned<Cow<'s, str>>>,
}
impl<'s> Transpose<'s> {
    pub fn validate(&mut self, diags: &Diagnostics) {
        score_helpers::check_part(diags, &self.part);
    }
}

#[derive(FromRawDirective)]
/// Change the base pitch of the current tuning for the named parts, or if no
/// parts are named, for the default tuning. If `absolute`, use the pitch as the
/// absolute base pitch. If `relative`, multiply the base pitch by the given
/// factor. Example: `set_base_pitch(relative="^1|12")` would transpose the
/// tuning up one 12-TET half step. Only one of `absolute` or `relative` may be
/// given.
pub struct SetBasePitch<'s> {
    pub span: Span,
    /// Set the base pitch of the current tuning to this absolute pitch value
    pub absolute: Option<Spanned<Pitch>>,
    /// Multiply the base pitch of the current tuning by the specified factor
    pub relative: Option<Spanned<Pitch>>,
    /// Which parts the tune; if not specified, all parts are tuned
    pub part: Vec<Spanned<Cow<'s, str>>>,
}
impl<'s> SetBasePitch<'s> {
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
pub struct ResetTuning<'s> {
    pub span: Span,
    /// Which parts the tune; if not specified, all parts are tuned
    pub part: Vec<Spanned<Cow<'s, str>>>,
}
impl<'s> ResetTuning<'s> {
    pub fn validate(&mut self, diags: &Diagnostics) {
        score_helpers::check_part(diags, &self.part);
    }
}

#[derive(FromRawDirective)]
/// Set the MIDI instrument number for zero or more parts. If no part is
/// specified, this becomes the default instrument for all parts without a
/// specific instrument. It is an error to name a part that doesn't appear
/// somewhere in the score.
pub struct MidiInstrument<'s> {
    pub span: Span,
    /// Midi instrument number from 1 to 128
    pub instrument: Spanned<u32>,
    /// Optional bank number from 1 to 16384
    pub bank: Option<Spanned<u32>>,
    /// Which parts use this instrument; if not specified, all unassigned parts
    /// use it
    pub part: Vec<Spanned<Cow<'s, str>>>,
}
impl<'s> MidiInstrument<'s> {
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
/// Set the CSound instrument number or name for zero or more parts. If no part
/// is specified, this becomes the default instrument for all parts without a
/// specific instrument. It is an error to name a part that doesn't appear
/// somewhere in the score. You must specify exactly one of number or name.
pub struct CsoundInstrument<'s> {
    pub span: Span,
    /// CSound instrument number
    pub number: Option<Spanned<u32>>,
    /// CSound instrument name
    pub name: Option<Spanned<Cow<'s, str>>>,
    /// Which parts use this instrument; if not specified, all unassigned parts
    /// use it
    pub part: Vec<Spanned<Cow<'s, str>>>,
}
impl<'s> CsoundInstrument<'s> {
    pub fn validate(&mut self, diags: &Diagnostics) {
        score_helpers::check_part(diags, &self.part);
        let n = [self.number.is_some(), self.name.is_some()]
            .into_iter()
            .fold(0usize, |x, v| x + if v { 1 } else { 0 });
        if n != 1 {
            diags.err(
                code::USAGE,
                self.span,
                "exactly one of 'number' or 'name' must be present",
            );
        }
    }
}

#[derive(FromRawDirective)]
/// Set tempo, with possible accelerando or ritardando (gradual change).
pub struct Tempo<'s> {
    pub _s: &'s (),
    pub span: Span,
    /// Tempo in beats per minute
    pub bpm: Spanned<Ratio<u32>>,
    /// Optional effective time relative to the current score time. This can be
    /// useful for inserting a tempo change part way through a score line.
    /// Defaults to 0.
    pub start_time: Option<Spanned<Ratio<u32>>>,
    /// Optional end tempo; if specified, duration is required. Indicates that
    /// the tempo should change gradually from `bpm` to `end_bpm` over
    /// `duration` beats.
    pub end_bpm: Option<Spanned<Ratio<u32>>>,
    /// Must appear with `end_bpm` to indicate the duration of a gradual tempo
    /// change.
    pub duration: Option<Spanned<Ratio<u32>>>,
}
impl<'s> Tempo<'s> {
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
/// Mark a moment in the score. The mark may be used for repeats or to generate
/// a subset of musical output. There are no restrictions around the placement
/// of marks, but there are restrictions on what marks may be used as repeat
/// delimiters. See the `repeat` directive.
pub struct Mark<'s> {
    pub span: Span,
    /// The mark's label
    pub label: Spanned<Cow<'s, str>>,
}
impl<'s> Mark<'s> {
    pub fn validate(&mut self, _diags: &Diagnostics) {}
}

#[derive(FromRawDirective)]
/// Repeat a section of the timeline delimited by two marks. The start mark must
/// strictly precede the end mark. No tied notes or pending dynamic changes may
/// be unresolved at the point of the end mark.
pub struct Repeat<'s> {
    pub span: Span,
    /// Label of mark at the beginning of the repeated section
    pub start: Spanned<Cow<'s, str>>,
    /// Label of mark at the end of the repeated section
    pub end: Spanned<Cow<'s, str>>,
    pub times: Option<Spanned<u32>>,
}
impl<'s> Repeat<'s> {
    pub fn validate(&mut self, _diags: &Diagnostics) {}
}

#[derive(FromRawDirective)]
/// Define an isomorphic mapping for a tuning. The mapping is placed into a
/// layout with the 'place_mapping' directive.
pub struct DefineIsomorphicMapping<'s> {
    pub span: Span,
    /// Name of mapping
    pub mapping: Spanned<Cow<'s, str>>,
    /// Scale; if omitted, use the current default scale
    pub scale: Option<Spanned<Cow<'s, str>>>,
    /// Number of scale degrees to go up in the horizontal direction
    pub steps_h: Spanned<u32>,
    /// Number of scale degrees to go up in the vertical or up-right direction
    pub steps_v: Spanned<u32>,
}
impl<'s> DefineIsomorphicMapping<'s> {
    pub fn validate(&mut self, _diags: &Diagnostics) {}
}

#[derive(FromRawDirective, Clone)]
/// Define a manual mapping of notes to keyboard positions. The mapping is
/// placed into a layout with the 'place_mapping' directive.
pub struct DefineManualMapping<'s> {
    pub span: Span,
    /// Name of mapping
    pub mapping: Spanned<Cow<'s, str>>,
    /// Scale; if omitted, use the current default scale
    pub scale: Option<Spanned<Cow<'s, str>>>,
    /// Factor to multiply by the pitches for horizontal tiling of the mapping;
    /// default is 1
    pub h_factor: Option<Spanned<Pitch>>,
    /// Factor to multiply by the pitches for vertical tiling of the mapping;
    /// default is 2
    pub v_factor: Option<Spanned<Pitch>>,
    pub layout_block: Spanned<LayoutBlock<'s>>,
}
impl<'s> DefineManualMapping<'s> {
    pub fn validate(&mut self, _diags: &Diagnostics) {}
}

#[derive(FromRawDirective)]
/// Place a mapping onto a layout for a keyboard.
pub struct PlaceMapping<'s> {
    pub span: Span,
    /// Name of layout
    pub layout: Spanned<Cow<'s, str>>,
    /// Name of mapping
    pub mapping: Spanned<Cow<'s, str>>,
    /// Base pitch; defaults to the base pitch of the default tuning
    pub base_pitch: Option<Spanned<Pitch>>,
    /// Name of keyboard
    pub keyboard: Spanned<Cow<'s, str>>,
    /// Row of the base note for isomorphic layouts or the anchor note for
    /// manual layouts
    pub anchor_row: Spanned<u32>,
    /// Column of the base note for isomorphic layouts or the anchor note for
    /// manual layouts
    pub anchor_col: Spanned<u32>,
    /// Number of rows *above* the anchor position to include in the region
    /// containing the mapping; default is to extend to the top of the keyboard.
    /// May be 0.
    pub rows_above: Option<Spanned<u32>>,
    /// Number of rows *below* the anchor position to include in the region
    /// containing the mapping; default is to extend to the bottom of the
    /// keyboard. May be 0.
    pub rows_below: Option<Spanned<u32>>,
    /// Number of columns to the *left* of the anchor position to include in the
    /// region containing the mapping; default is to extend to the leftmost
    /// column of the keyboard. May be 0.
    pub cols_left: Option<Spanned<u32>>,
    /// Number of columns to the *right* of the anchor position to include in
    /// the region containing the mapping; default is to extend to the rightmost
    /// column of the keyboard. May be 0.
    pub cols_right: Option<Spanned<u32>>,
}
impl<'s> PlaceMapping<'s> {
    pub fn validate(&mut self, _diags: &Diagnostics) {}
}

#[derive(FromRawDirective)]
pub enum Directive<'s> {
    Syntoniq(Syntoniq<'s>),
    DefineScale(DefineScale<'s>),
    DefineGeneratedScale(DefineGeneratedScale<'s>),
    UseScale(UseScale<'s>),
    Transpose(Transpose<'s>),
    SetBasePitch(SetBasePitch<'s>),
    ResetTuning(ResetTuning<'s>),
    MidiInstrument(MidiInstrument<'s>),
    CsoundInstrument(CsoundInstrument<'s>),
    Tempo(Tempo<'s>),
    Mark(Mark<'s>),
    Repeat(Repeat<'s>),
    DefineIsomorphicMapping(DefineIsomorphicMapping<'s>),
    DefineManualMapping(DefineManualMapping<'s>),
    PlaceMapping(PlaceMapping<'s>),
}
