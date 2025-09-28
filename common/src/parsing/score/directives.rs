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
/// Set the syntoniq file format version. This must be the first functional item in the file.
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
/// Define a scale. The scale called "default" is pre-defined and corresponds to 12-EDO.
pub struct DefineScale {
    pub span: Span,
    /// scale name
    pub name: Spanned<String>,
    /// base pitch in pitch syntax; default is 220^1|4 (middle C from A-440 in 12-EDO)
    pub base_pitch: Option<Spanned<Pitch>>,
    /// ratio to be applied by the octave marker; default is 2 (one octave)
    pub cycle_ratio: Option<Spanned<Ratio<u32>>>,
}
impl DefineScale {
    pub fn validate(&mut self, _diags: &Diagnostics) {}
}

#[derive(FromRawDirective)]
pub enum Directive {
    Syntoniq(Syntoniq),
    DefineScale(DefineScale),
}
