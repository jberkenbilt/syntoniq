use std::fmt::{Display, Formatter};
use std::ops::Index;
use std::ops::Range;
use std::sync::Mutex;

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

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub span: Span,
    pub message: String,
}

#[derive(Default, Debug)]
pub struct Diagnostics {
    pub list: Mutex<Vec<Diagnostic>>,
}
impl Diagnostics {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Display for Diagnostics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let guard = self.list.lock().unwrap();
        if guard.is_empty() {
            return writeln!(f, "no errors");
        }
        write!(f, "ERRORS:")?;
        for i in &*guard {
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
        self.list.lock().unwrap().push(Diagnostic {
            code,
            span: span.into(),
            message: msg.into(),
        });
    }

    pub fn has_errors(&self) -> bool {
        !self.list.lock().unwrap().is_empty()
    }

    pub fn get_all(&self) -> Vec<Diagnostic> {
        self.list.lock().unwrap().clone()
    }
}

// TODO: used?
pub fn line_starts(src: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(src.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}
