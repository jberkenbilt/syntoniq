use crate::parsing::diagnostics::{Diagnostic, Diagnostics, code};
use crate::parsing::model::{Identifier, NoteOctave, Param, ParamValue, Span, Spanned};
use crate::pitch::Pitch;
use num_rational::Ratio;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, RwLock};
use std::{mem, ptr};

struct ArcPtr {
    p: *mut (),
    delete: Box<dyn Fn(*mut ())>,
}
#[derive(Default)]
pub struct ArcContext {
    converted: HashMap<*const (), Option<ArcPtr>>,
}
impl Drop for ArcContext {
    fn drop(&mut self) {
        for v in mem::take(&mut self.converted).into_values().flatten() {
            (*v.delete)(v.p);
        }
    }
}

pub trait ToStatic<'s> {
    type Static;
    fn to_static(&self, arc_context: &mut ArcContext) -> Self::Static;
}

impl<'s> ToStatic<'s> for bool {
    type Static = bool;
    fn to_static(&self, _: &mut ArcContext) -> Self::Static {
        *self
    }
}

impl<'s> ToStatic<'s> for i32 {
    type Static = i32;
    fn to_static(&self, _: &mut ArcContext) -> Self::Static {
        *self
    }
}

impl<'s> ToStatic<'s> for u32 {
    type Static = u32;
    fn to_static(&self, _: &mut ArcContext) -> Self::Static {
        *self
    }
}

impl<'s> ToStatic<'s> for usize {
    type Static = usize;
    fn to_static(&self, _: &mut ArcContext) -> Self::Static {
        *self
    }
}

impl<'s, T: Copy> ToStatic<'s> for Ratio<T> {
    type Static = Ratio<T>;

    fn to_static(&self, _: &mut ArcContext) -> Self::Static {
        *self
    }
}

impl<'s> ToStatic<'s> for AtomicI32 {
    type Static = AtomicI32;
    fn to_static(&self, _: &mut ArcContext) -> Self::Static {
        AtomicI32::new(self.load(Ordering::Relaxed))
    }
}

impl<'s, T> ToStatic<'s> for Arc<T>
where
    T: ToStatic<'s>,
{
    type Static = Arc<T::Static>;

    fn to_static(&self, arc_context: &mut ArcContext) -> Self::Static {
        let from_ptr = self.as_ref() as *const _ as *const ();
        let entry = match arc_context.converted.entry(from_ptr) {
            Entry::Occupied(e) => {
                let Some(p) = e.get().as_ref() else {
                    panic!("loop_detected in to_static");
                };
                p.p as *const Arc<T::Static>
            }
            Entry::Vacant(v) => {
                // Can't allocate and insert here since it would require two mutable references
                // to arc_context.
                v.insert(None);
                ptr::null_mut()
            }
        };
        let to_ptr = if entry.is_null() {
            let new = Arc::new(self.as_ref().to_static(arc_context));
            // Create a raw pointer to one copy of the Arc. This keeps the Arc alive. This will
            // get cleaned up by the Drop implementation for ArcContext.
            let p = Box::into_raw(Box::new(new));
            let ap = ArcPtr {
                p: p as *mut _,
                delete: Box::new(|p| {
                    drop(unsafe { Box::from_raw(p as *mut Arc<T::Static>) });
                }),
            };
            arc_context.converted.insert(from_ptr, Some(ap));
            p as *const _
        } else {
            entry
        };

        // Safety: at this moment, we know to_ptr is a non-null pointer to memory that was taken
        // from a Box<Arc<T::Static>>. Taking it as a ref and cloning it will make a new copy of
        // the Arc.
        unsafe { to_ptr.as_ref() }.unwrap().clone()
    }
}

impl<'s, T> ToStatic<'s> for Option<T>
where
    T: ToStatic<'s>,
{
    type Static = Option<T::Static>;

    fn to_static(&self, arc_context: &mut ArcContext) -> Self::Static {
        self.as_ref().map(|x| x.to_static(arc_context))
    }
}

impl<'s, T> ToStatic<'s> for RwLock<T>
where
    T: ToStatic<'s>,
{
    type Static = RwLock<T::Static>;

    fn to_static(&self, arc_context: &mut ArcContext) -> Self::Static {
        RwLock::new(self.read().unwrap().to_static(arc_context))
    }
}

impl<'s> ToStatic<'s> for Cow<'s, str> {
    type Static = Cow<'static, str>;

    fn to_static(&self, _: &mut ArcContext) -> Self::Static {
        Cow::Owned(self.to_string())
    }
}

impl<'s, K, V> ToStatic<'s> for BTreeMap<K, V>
where
    K: Eq + Hash + ToStatic<'s>,
    K::Static: Ord + Eq + Hash,
    V: ToStatic<'s>,
{
    type Static = BTreeMap<K::Static, V::Static>;

    fn to_static(&self, arc_context: &mut ArcContext) -> Self::Static {
        self.iter()
            .map(|(k, v)| (k.to_static(arc_context), v.to_static(arc_context)))
            .collect()
    }
}

impl<'s, T> ToStatic<'s> for Vec<T>
where
    T: ToStatic<'s>,
{
    type Static = Vec<T::Static>;

    fn to_static(&self, arc_context: &mut ArcContext) -> Self::Static {
        self.iter().map(|x| x.to_static(arc_context)).collect()
    }
}

pub trait CheckValue<'s>: Sized {
    fn check_value(pv: &ParamValue<'s>) -> Result<Self, impl AsRef<str>>;
}

pub fn check_value<'s, T>(
    diags: &Diagnostics,
    d_name: &Identifier<'s>,
    p: &Param<'s>,
) -> Option<Spanned<T>>
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
        pv.try_as_pitch().cloned().ok_or("should be a pitch")
    }
}

impl<'s> CheckValue<'s> for Cow<'s, str> {
    fn check_value(pv: &ParamValue<'s>) -> Result<Self, impl AsRef<str>> {
        pv.try_as_string().cloned().ok_or("should be a string")
    }
}

impl<'s> CheckValue<'s> for NoteOctave<'s> {
    fn check_value(pv: &ParamValue<'s>) -> Result<Self, impl AsRef<str>> {
        pv.try_as_note().cloned().ok_or("should be a note name")
    }
}

impl<'s> CheckValue<'s> for Identifier<'s> {
    fn check_value(pv: &ParamValue<'s>) -> Result<Self, impl AsRef<str>> {
        pv.try_as_identifier()
            .cloned()
            .ok_or("should be an identifier")
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

pub fn format_note_cycle<'s>(note_name: Cow<'s, str>, cycle: i32) -> Cow<'s, str> {
    match cycle {
        1 => Cow::Owned(format!("{note_name}'")),
        -1 => Cow::Owned(format!("{note_name},")),
        x if x > 1 => Cow::Owned(format!("{note_name}'{x}")),
        x if x < -1 => Cow::Owned(format!("{note_name},{}", -x)),
        _ => note_name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_static() {
        // Tests pass with valgrind. The implementation was done with no AI help and passed AI code
        // review.
        let a1 = Arc::new(3);
        let a2 = a1.clone();
        assert_eq!(Arc::strong_count(&a1), 2);
        assert!(ptr::eq(a1.as_ref(), a2.as_ref()));
        let mut arc_context = ArcContext::default();
        let a3: Arc<i32> = a1.to_static(&mut arc_context);
        let a4: Arc<i32> = a2.to_static(&mut arc_context);
        let a5: Arc<i32> = a1.clone().to_static(&mut arc_context);
        assert!(!ptr::eq(a1.as_ref(), a3.as_ref()));
        assert!(ptr::eq(a3.as_ref(), a4.as_ref()));
        assert!(ptr::eq(a3.as_ref(), a5.as_ref()));
        assert_eq!(Arc::strong_count(&a1), 2);
        assert_eq!(Arc::strong_count(&a3), 4);
        drop(arc_context);
        assert_eq!(Arc::strong_count(&a1), 2);
        assert_eq!(Arc::strong_count(&a3), 3);
    }
}
