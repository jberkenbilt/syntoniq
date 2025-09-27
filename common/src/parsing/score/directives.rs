use crate::parsing::diagnostics::{Diagnostics, code};
use crate::parsing::model::{Span, Spanned};
use crate::parsing::score::FromRawDirective;
use crate::parsing::score::HashSet;
use crate::parsing::score::RawDirective;
use crate::parsing::score_helpers;
use crate::pitch::Pitch;
use directive_derive::FromRawDirective;
use num_rational::Ratio;

#[derive(FromRawDirective)]
pub struct Syntoniq {
    pub span: Span,
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
pub struct DefineScale {
    pub span: Span,
    pub name: Spanned<String>,
    pub base_pitch: Option<Spanned<Pitch>>,
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
