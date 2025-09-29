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
    pub const EMPTY_FILE: &str = "E1005 empty file";
    pub const NUM_FORMAT: &str = "E1006 incorrect number format";
    pub const PITCH_SYNTAX: &str = "E1007 incorrect pitch syntax";
    pub const NOTE_SYNTAX: &str = "E1008 incorrect note syntax";
    pub const SCORE_SYNTAX: &str = "E1009 incorrect score syntax";
    pub const DYNAMIC_SYNTAX: &str = "E1010 incorrect dynamic syntax";
    pub const TOPLEVEL_SYNTAX: &str = "E1011 incorrect syntax";
    pub const DIRECTIVE_SYNTAX: &str = "E1012 incorrect directive syntax";
    pub const SCALE_SYNTAX: &str = "E1013 incorrect scale block syntax";
    pub const UNKNOWN_DIRECTIVE: &str = "E1014 unknown directive";
    pub const UNKNOWN_DIRECTIVE_PARAM: &str = "E1015 unknown directive parameter";
    pub const INCORRECT_DIRECTIVE_PARAM: &str = "E1016 incorrect parameter type";
    pub const DIRECTIVE_USAGE: &str = "E1017 incorrect directive usage";
    pub const SCALE: &str = "E1018 incorrect scale data";
    pub const INITIALIZATION: &str = "E1019 syntoniq initialization";
    pub const USAGE: &str = "E1020 general usage";
    pub const SCORE: &str = "E1021 incorrect score block";
    pub const TUNE: &str = "E1022 incorrect tuning";
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
    pub seen: RefCell<HashSet<(&'static str, Spanned<String>)>>,
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
            for c in &i.context {
                write!(
                    f,
                    ", (context {}..{}: {})",
                    c.span.start, c.span.end, c.value
                )?;
            }
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
        if self.seen.borrow_mut().insert((d.code, d.message.clone())) {
            self.list.borrow_mut().push(d)
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.list.borrow_mut().is_empty()
    }

    pub fn num_errors(&self) -> usize {
        self.list.borrow().len()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let x = Diagnostics::new();
        assert!(x.to_string().contains("no errors"));
        x.push(Diagnostic::new("e1", 1..2, "something").with_context(2..3, "else"));
        x.push(Diagnostic::new("e2", 3..4, "potato").with_context(4..5, "salad"));
        assert_eq!(
            x.to_string(),
            "ERRORS: offset 1..2: e1: something, (context 2..3: else), offset 3..4: e2: potato, (context 4..5: salad)"
        );
    }
}
