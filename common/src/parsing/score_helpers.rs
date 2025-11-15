use crate::parsing::diagnostics::{Diagnostic, Diagnostics, code};
use crate::parsing::model::{Param, ParamValue, Span, Spanned};
use crate::pitch::Pitch;
use num_rational::Ratio;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::hash::Hash;

pub trait CheckValue<'s>: Sized {
    fn check_value(p: &ParamValue<'s>) -> Result<Self, impl AsRef<str>>;
}

pub fn check_value<'s, T>(diags: &Diagnostics, d_name: &str, p: &Param<'s>) -> Option<Spanned<T>>
where
    T: CheckValue<'s> + Serialize + Debug,
{
    let k = &p.key;
    let v = &p.value;
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

impl<'s> CheckValue<'s> for u32 {
    fn check_value(pv: &ParamValue) -> Result<Self, impl AsRef<str>> {
        pv.try_as_int().ok_or("should be an integer")
    }
}

impl<'s> CheckValue<'s> for Ratio<u32> {
    fn check_value(pv: &ParamValue) -> Result<Self, impl AsRef<str>> {
        pv.try_as_ratio()
            .ok_or("should be a rational number or decimal")
    }
}

impl<'s> CheckValue<'s> for Pitch {
    fn check_value(pv: &ParamValue) -> Result<Self, impl AsRef<str>> {
        pv.try_as_pitch().cloned().ok_or("should be an pitch")
    }
}

impl<'s> CheckValue<'s> for Cow<'s, str> {
    fn check_value(pv: &ParamValue<'s>) -> Result<Self, impl AsRef<str>> {
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

pub fn check_part(diags: &Diagnostics, items: &[Spanned<Cow<'_, str>>]) {
    check_unique(diags, items);
    for i in items {
        if i.value.is_empty() {
            diags.err(code::USAGE, i.span, "a part name may not be empty");
        }
    }
}

pub fn check_duplicate_by_part<'s, T: Clone>(
    diags: &Diagnostics,
    thing: &str,
    parts: &[Spanned<Cow<'s, str>>],
    span: Span,
    existing: &mut HashMap<Cow<'s, str>, Span>,
    item: T,
    map: &mut BTreeMap<Cow<'s, str>, T>,
) {
    let part_spans = if parts.is_empty() {
        vec![Spanned::new(span, Cow::Borrowed(""))]
    } else {
        parts.to_vec()
    };
    for part_span in part_spans {
        if let Some(old) = existing.insert(part_span.value.clone(), part_span.span) {
            let what = if part_span.value.is_empty() {
                format!("default {thing}")
            } else {
                format!("{thing} for part '{}'", part_span.value)
            };
            diags.push(
                Diagnostic::new(
                    code::MIDI,
                    span,
                    format!("a {what} has already been specified"),
                )
                .with_context(old, "here is the previous occurrence"),
            );
        } else {
            map.insert(part_span.value, item.clone());
        }
    }
}
