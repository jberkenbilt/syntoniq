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
/// Set tuning for some voices. At most one of base_pitch, base_factor, or
/// base_note may be specified. If no parts are specified, this changes the
/// tuning used for parts that have no specified tuning.
pub struct Tune {
    pub span: Span,
    /// The name of the scale
    pub scale: Spanned<String>,
    /// Which parts the tune; if not specified, all parts are tuned
    pub part: Vec<Spanned<String>>,
    /// Set the absolute base pitch of the scale
    pub base_pitch: Option<Spanned<Pitch>>,
    /// Set the base pitch of the scale as a factor of the prior tuning's base pitch
    pub base_factor: Option<Spanned<Pitch>>,
    /// Set the base pitch of the scale to the given note in the prior tuning
    pub base_note: Option<Spanned<String>>,
}
impl Tune {
    pub fn validate(&mut self, diags: &Diagnostics) {
        score_helpers::check_unique(diags, &self.part);
        let n = [
            self.base_pitch.is_some(),
            self.base_factor.is_some(),
            self.base_note.is_some(),
        ]
        .into_iter()
        .fold(0usize, |x, v| x + if v { 1 } else { 0 });
        if n > 1 {
            diags.err(
                code::TUNE,
                self.span,
                "at most one of base_pitch, base_factor, or base_note may be specified",
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
        score_helpers::check_unique(diags, &self.part);
    }
}

#[derive(FromRawDirective)]
pub enum Directive {
    Syntoniq(Syntoniq),
    DefineScale(DefineScale),
    Tune(Tune),
    ResetTuning(ResetTuning),
}
