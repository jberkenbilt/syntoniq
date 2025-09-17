use crate::pitch::Pitch;
use num_rational::Ratio;
use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Index;
use std::ops::Range;
use std::sync::LazyLock;
use std::{env, mem};

pub mod code {
    pub const LEXICAL: &str = "E1001 lexical error";
    pub const SYNTAX: &str = "E1002 syntax error";
    pub const NUMBER: &str = "E1003 numerical error";
    pub const PITCH: &str = "E1004 pitch error";
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
impl Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.start..index.end]
    }
}
impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}
impl Span {
    pub fn relative_to(&self, other: Span) -> Span {
        Span {
            start: self.start - other.start,
            end: self.end - other.start,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token<'s, T: Debug> {
    pub(crate) raw: &'s str,
    pub(crate) t: T,
}
impl<'s, T: Debug> Token<'s, T> {
    pub fn new_spanned(raw: &'s str, span: impl Into<Span>, t: T) -> Spanned<Self> {
        Spanned::new(span, Self { raw, t })
    }
}
impl<'s, T: Debug + Copy> Copy for Token<'s, T> {}

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T: Debug> {
    pub span: Span,
    pub value: T,
}
impl<T: Debug + Copy> Copy for Spanned<T> {}

impl<T: Debug> Spanned<T> {
    pub fn new(span: impl Into<Span>, value: impl Into<T>) -> Self {
        Self {
            span: span.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub span: Span,
    pub message: String,
}

#[derive(Default, Debug)]
pub struct Diagnostics {
    pub list: RefCell<Vec<Diagnostic>>,
}
impl Diagnostics {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Display for Diagnostics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let list = self.list.borrow_mut();
        if list.is_empty() {
            return writeln!(f, "no errors");
        }
        write!(f, "ERRORS:")?;
        for i in &*list {
            write!(
                f,
                "\n  {}..{}: {}: {}",
                i.span.start, i.span.end, i.code, i.message
            )?;
        }
        Ok(())
    }
}
impl Diagnostics {
    pub fn err(&self, code: &'static str, span: impl Into<Span>, msg: impl Into<String>) {
        self.list.borrow_mut().push(Diagnostic {
            code,
            span: span.into(),
            message: msg.into(),
        });
    }

    pub fn has_errors(&self) -> bool {
        !self.list.borrow_mut().is_empty()
    }

    pub fn get_all(&self) -> Vec<Diagnostic> {
        mem::take(&mut self.list.borrow_mut())
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Represents a pitch. All ratios also parse into pitches. If something that parsed into
/// a pitch was originally specified as a ratio, you can get the value as a ratio. During the
/// lexical phase, we never know whether a ratio is supposed to be a ratio or a pitch. During the
/// semantic phase, if a pitch was provided when a ratio was wanted, we can give an error at that
/// time.
pub enum PitchOrRatio {
    Ratio((Ratio<u32>, Pitch)),
    Pitch(Pitch),
}

impl PitchOrRatio {
    pub fn into_pitch(self) -> Pitch {
        match self {
            PitchOrRatio::Ratio((_, p)) => p,
            PitchOrRatio::Pitch(p) => p,
        }
    }

    pub fn try_into_ratio(self) -> Option<Ratio<u32>> {
        match self {
            PitchOrRatio::Ratio((r, _)) => Some(r),
            PitchOrRatio::Pitch(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    PitchOrRatio(PitchOrRatio),
    String(String),
}

impl ParamValue {
    pub fn try_into_pitch(self) -> Option<Pitch> {
        match self {
            ParamValue::PitchOrRatio(pr) => Some(pr.into_pitch()),
            ParamValue::String(_) => None,
        }
    }

    pub fn try_into_ratio(self) -> Option<Ratio<u32>> {
        match self {
            ParamValue::PitchOrRatio(pr) => pr.try_into_ratio(),
            ParamValue::String(_) => None,
        }
    }

    pub fn try_into_string(self) -> Option<String> {
        match self {
            ParamValue::PitchOrRatio(_r) => None,
            ParamValue::String(s) => Some(s),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub key: Spanned<String>,
    pub value: Spanned<ParamValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Directive {
    pub name: Spanned<String>,
    pub params: Vec<Param>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicLeader {
    pub name: Spanned<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoteLeader {
    pub name: Spanned<String>,
    pub note: Spanned<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteOption {
    Accent,
    Marcato,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteBehavior {
    Sustain,
    Slide,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegularNote {
    pub duration: Option<Spanned<Ratio<u32>>>,
    pub name: Spanned<String>,
    pub octave: Option<Spanned<i8>>,
    pub options: Vec<Spanned<NoteOption>>,
    pub behavior: Option<Spanned<NoteBehavior>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hold {
    pub duration: Option<Spanned<Ratio<u32>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Note {
    Regular(RegularNote),
    Hold(Hold),
    BarCheck(Span),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DynamicChange {
    Crescendo,
    Diminuendo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dynamic {
    pub level: Spanned<u8>,
    pub change: Option<Spanned<DynamicChange>>,
    pub position: Spanned<Ratio<u32>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicLine {
    pub leader: DynamicLeader,
    pub dynamics: Vec<Spanned<Dynamic>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoteLine {
    pub leader: NoteLeader,
    pub notes: Vec<Spanned<Note>>,
}

pub fn trace(msg: impl Display) {
    static TRACING: LazyLock<bool> = LazyLock::new(|| env::var("SYNTONIQ_TRACE_LEXER").is_ok());
    if *TRACING {
        eprintln!("{msg}");
    }
}

/// Helper for merge_option_spans
pub trait GetSpan {
    fn get_span(&self) -> Option<Span>;
}

impl<T: Debug> GetSpan for Spanned<T> {
    fn get_span(&self) -> Option<Span> {
        Some(self.span)
    }
}

impl<T: Debug> GetSpan for Option<Spanned<T>> {
    fn get_span(&self) -> Option<Span> {
        self.as_ref().map(|x| x.span)
    }
}

impl<T: Debug> GetSpan for &[Spanned<T>] {
    fn get_span(&self) -> Option<Span> {
        if self.is_empty() {
            return None;
        }
        Some((self[0].span.start..self[self.len() - 1].span.end).into())
    }
}

impl<T: Debug> GetSpan for Option<Vec<Spanned<T>>> {
    fn get_span(&self) -> Option<Span> {
        let s = self.as_ref()?.as_slice();
        s.get_span()
    }
}

/// Return a span that covers the range of all these spans assuming they are sorted.
pub(crate) fn merge_spans(spans: &[Option<Span>]) -> Option<Span> {
    let first = spans.iter().find(|x| x.is_some())?.as_ref()?;
    let last = spans.iter().rfind(|x| x.is_some())?.as_ref()?;
    Some((first.start..last.end).into())
}

// TODO: used?
pub(crate) fn _line_starts(src: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(src.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_spans() {
        assert_eq!(
            GetSpan::get_span(&Some(vec![
                Spanned::<i32>::new(0..4, 3),
                Spanned::new(5..8, 6),
            ]))
            .unwrap(),
            (0..8).into()
        );
        assert!(merge_spans(&[]).is_none());
        assert!(merge_spans(&[None]).is_none());
        assert_eq!(
            merge_spans(&[None, Some((1..2).into()), None, Some((3..4).into()), None]).unwrap(),
            (1..4).into()
        );
    }
}
