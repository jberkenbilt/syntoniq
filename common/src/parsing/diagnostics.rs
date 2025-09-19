use crate::parsing::model::{code, Span, Spanned};
use annotate_snippets::renderer::DecorStyle;
use annotate_snippets::{AnnotationKind, Group, Level, Renderer, Snippet};
use serde::Serialize;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::mem;

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
        // TODO: improve this
        for i in &*list {
            write!(
                f,
                "\n  {}..{}: {}: {}",
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
        self.list.borrow_mut().push(d)
    }

    pub fn has_errors(&self) -> bool {
        !self.list.borrow_mut().is_empty()
    }

    pub fn get_all(&self) -> Vec<Diagnostic> {
        mem::take(&mut self.list.borrow_mut())
    }

    pub fn render(&self, filename: &str, src: &str) {
        let list = self.list.borrow();
        let report: Vec<Group> = list.iter().map(|x| x.group(filename, src)).collect();
        let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
        anstream::eprintln!("{}", renderer.render(&report));
    }
}
