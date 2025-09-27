use crate::parsing::diagnostics::{Diagnostics, code};
use crate::parsing::model::{Param, ParamValue, RawDirective, Spanned};
use crate::pitch::Pitch;
use num_rational::Ratio;
use serde::Serialize;
use std::fmt::Debug;

pub trait FromRawDirective: Sized {
    fn from_raw(diags: &Diagnostics, d: &RawDirective) -> Option<Self>;
}

pub trait CheckValue: Sized {
    fn check_value(p: &ParamValue) -> Result<Self, impl AsRef<str>>;
}

pub fn check_value<T>(diags: &Diagnostics, d_name: &str, p: &Param) -> Option<Spanned<T>>
where
    T: CheckValue + Serialize + Debug,
{
    let k = &p.kv.key;
    let v = &p.kv.value;
    match T::check_value(&v.value) {
        Ok(x) => Some(Spanned::new(v.span, x)),
        Err(msg) => {
            diags.err(
                code::INCORRECT_DIRECTIVE_PARAM,
                v.span,
                format!("'{d_name}': '{}' {}", k.value, msg.as_ref()),
            );
            None
        }
    }
}

impl CheckValue for u32 {
    fn check_value(pv: &ParamValue) -> Result<Self, impl AsRef<str>> {
        pv.try_as_int().ok_or("should be an integer")
    }
}

impl CheckValue for Ratio<u32> {
    fn check_value(pv: &ParamValue) -> Result<Self, impl AsRef<str>> {
        pv.try_as_ratio()
            .ok_or("should be a rational number or decimal")
    }
}

impl CheckValue for Pitch {
    fn check_value(pv: &ParamValue) -> Result<Self, impl AsRef<str>> {
        pv.try_as_pitch().cloned().ok_or("should be an pitch")
    }
}

impl CheckValue for String {
    fn check_value(pv: &ParamValue) -> Result<Self, impl AsRef<str>> {
        pv.try_as_string().cloned().ok_or("should be a string")
    }
}
