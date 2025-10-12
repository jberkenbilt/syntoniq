use crate::parsing::diagnostics::{Diagnostic, Diagnostics, code};
use crate::parsing::model::{Param, ParamValue, Spanned};
use crate::pitch::Pitch;
use num_rational::Ratio;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

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

pub fn check_unique<T: Debug + Serialize + Eq + Hash>(diags: &Diagnostics, items: &[Spanned<T>]) {
    let mut seen = HashMap::new();
    for i in items {
        if let Some(old) = seen.insert(&i.value, i.span) {
            diags.push(
                Diagnostic::new(code::USAGE, i.span, "this value has already been used")
                    .with_context(old, "here is the previous value"),
            );
        }
    }
}

pub fn check_part(diags: &Diagnostics, items: &[Spanned<String>]) {
    check_unique(diags, items);
    for i in items {
        if i.value.is_empty() {
            diags.err(code::USAGE, i.span, "a part name may not be empty");
        }
    }
}
