use crate::parsing::model::Span;
use serde::Serialize;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::mem;

// TODO: used?
pub(crate) fn _line_starts(src: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(src.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub span: Span,
    pub message: String,
}

#[derive(Serialize, Default, Debug)]
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
