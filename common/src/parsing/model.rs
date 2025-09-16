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

#[derive(Debug, Clone)]
pub struct SpannedToken<'s, T: Debug> {
    pub(crate) span: Span,
    pub(crate) data: &'s str,
    pub(crate) t: T,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T: Debug> {
    span: Span,
    value: T,
}

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

pub fn trace(msg: impl Display) {
    static TRACING: LazyLock<bool> = LazyLock::new(|| env::var("SYNTONIQ_TRACE_LEXER").is_ok());
    if *TRACING {
        eprintln!("{msg}");
    }
}

pub(crate) fn make_spanned<'s, I, T: Debug>(
    input: &'s str,
    t: T,
) -> impl FnOnce((I, Range<usize>)) -> SpannedToken<'s, T> {
    move |(_, span)| SpannedToken {
        data: &input[span.clone()],
        span: span.into(),
        t,
    }
}

pub(crate) fn merge_span<T: Debug>(tokens: &[SpannedToken<T>]) -> Span {
    if tokens.is_empty() {
        0..1
    } else {
        tokens[0].span.start..tokens[tokens.len() - 1].span.end
    }
    .into()
}

// TODO: used?
pub(crate) fn _line_starts(src: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(src.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}
