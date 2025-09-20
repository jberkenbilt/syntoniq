use crate::parsing::model::{Span, Spanned};
use annotate_snippets::renderer::DecorStyle;
use annotate_snippets::{AnnotationKind, Group, Level, Renderer, Snippet};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::mem;

pub const SYNTAX_ERROR: &str = "this syntax is not expected here";
pub mod code {
    pub const SYNTAX: &str = "E1001 syntax error";
    pub const NUM_RANGE: &str = "E1002 numeric range";
    pub const STRING: &str = "E1003 invalid string literal";
    pub const LINE_START: &str = "E1004 unable to infer line type";
    pub const EMPTY: &str = "E1005 empty file";
    pub const NUM_FORMAT: &str = "E1006 incorrect number format";
    pub const PITCH: &str = "E1007 incorrect pitch syntax";
    pub const NOTE: &str = "E1008 incorrect note syntax";
    pub const SCORE_SYNTAX: &str = "E1009 incorrect score syntax";
    pub const DYNAMIC: &str = "E1010 incorrect dynamic syntax";
    pub const TOPLEVEL_SYNTAX: &str = "E1011 incorrect syntax";
    pub const DIRECTIVE: &str = "E1012 incorrect directive syntax";
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: Spanned<String>,
    pub context: Vec<Spanned<String>>,
}
impl Diagnostic {
    pub fn new(code: &'static str, span: impl Into<Span>, msg: impl Into<String>) -> Self {
        Self {
            code,
            message: Spanned::new(span, msg),
            context: Default::default(),
        }
    }

    pub fn with_context(mut self, span: impl Into<Span>, msg: impl Into<String>) -> Self {
        self.context.push(Spanned::new(span, msg));
        self
    }

    pub fn group<'a>(&'a self, filename: &'a str, src: &'a str) -> Group<'a> {
        let mut source = Snippet::source(src).path(filename).annotation(
            AnnotationKind::Primary
                .span(self.message.span.into())
                .label(&self.message.value),
        );
        for m in &self.context {
            source = source.annotation(AnnotationKind::Context.span(m.span.into()).label(&m.value));
        }
        Level::ERROR.primary_title(self.code).element(source)
    }
}

#[derive(Serialize, Default, Debug)]
pub struct Diagnostics {
    pub list: RefCell<Vec<Diagnostic>>,
    #[serde(skip)]
    pub seen: RefCell<HashSet<(&'static str, Span)>>,
}
impl Diagnostics {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Display for Diagnostics {
    /// Diagnostics can be formatted as a string, but it's better to use [Diagnostics::render].
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let list = self.list.borrow_mut();
        if list.is_empty() {
            return writeln!(f, "no errors");
        }
        let mut first = true;
        for i in &*list {
            if first {
                write!(f, "ERRORS: ")?;
                first = false;
            } else {
                write!(f, ", ")?;
            }
            write!(
                f,
                "offset {}..{}: {}: {}",
                i.message.span.start, i.message.span.end, i.code, i.message.value
            )?;
        }
        Ok(())
    }
}
impl Diagnostics {
    /// Convenience function for adding a simple error without context
    pub fn err(&self, code: &'static str, span: impl Into<Span>, msg: impl Into<String>) {
        self.push(Diagnostic::new(code, span, msg))
    }

    pub fn push(&self, d: Diagnostic) {
        if self.seen.borrow_mut().insert((d.code, d.message.span)) {
            self.list.borrow_mut().push(d)
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.list.borrow_mut().is_empty()
    }

    pub fn get_all(&self) -> Vec<Diagnostic> {
        mem::take(&mut self.list.borrow_mut())
    }

    pub fn render(&self, filename: &str, src: &str) -> String {
        let list = self.list.borrow();
        let report: Vec<Group> = list.iter().map(|x| x.group(filename, src)).collect();
        let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
        renderer.render(&report)
    }
}
