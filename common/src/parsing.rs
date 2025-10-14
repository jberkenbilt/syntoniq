// Rust 1.89.0 is giving false positive on needless lifetimes.
#![allow(clippy::needless_lifetimes)]

// HOW TO UNDERSTAND THIS PARSING
//
// This code parses using the `winnow` parser combinator library. It uses winnow in a particular
// way. To get a basic understanding, read through the winnow tutorial, but just skim the section on
// error handling as this code does something a little different. At initial writing, winnow was at
// version 0.7.
//
// In a nutshell, winnow in particular, and parser combinators in general, work as follows:
//
// Start with a mutable input slice of an immutable input buffer. Everything contains references to
// the input buffer by creating slices of it.
//
// Repeatedly pass a mutable input slice to a parser. The parser's job is to consume zero or more
// tokens according to rules and to return a Result that is Ok if its rules all passed or Err
// otherwise. The most basic kinds of parsers just consume tokens. Consuming a token involves
// advancing the mutable input slice by one input value. A parser calls combinators that consume
// tokens according to some conditions. If all the rules match, it returns Ok. If not, it returns an
// error indicating one of three conditions: `Incomplete`, for streaming parsers to indicate that
// EOF was encountered while parsing (we don't use these), `Backtrack` indicating that the parser
// didn't match, or `Cut`, indicating that the rules didn't match but that some error was
// encountered that should preclude backtracking...but this is subtle.
//
// For this to work, there are certain combinators that try a list of alternative parsers. If one
// fails with a Backtrack error, it backtracks the input to the point where it was before the parser
// was called and tries the next parser. The `alt` parser is the main example of this. But this gets
// really tricky when parsers nest as a parser may end up consuming some tokens before ultimately
// backtracking to some intermediate point. That is admittedly a vague statement, partly because
// trying to fully understand and describe the exact behavior is tricky. Mostly it's best to just
// avoid this.
//
// Ultimately, a parser combinator is a backtracking parser whose job is to consider which of
// several branches is the right branch. This means the parser is always first deciding, "Is it this
// branch?" and then deciding, "Is this branch correct?" The balancing act is to make a clear
// separation between these concerns. Our parsers follow this broad pattern:
//
// - Do some kind of lookahead, either with `peek` or explicitly looking at the input stream, to
//   decide which branch we want to take
// - In the branch, consume tokens liberally so that, when the "is it this branch?" question was
//   answered affirmatively, we have a pretty good chance of consuming all the tokens for the
//   branch...but not always. In some cases, the parser fails to match even if we initially
//   committed to a particular branch. In this case, we fall back to a "degraded mode" in which we
//   grab tokens in a way that is much more likely to succeed, and then resume normal parsing after
//   some synchronization point. This is quite similar to normal error handling in a parser.
// - After the tokens have been consumed, rather than using `Cut`, perform validations and report
//   errors with our own `Diagnostics` system. This is a departure from how winnow documentation
//   describes how to do error handling, but I have found it to be much easier to reason about and
//   to create good, contextual error messages.
//
// At the end of each "pass" of parsing, if any diagnostics were issued, we stop. Otherwise, we
// continue with the next pass.
//
// The passes roughly correspond to lexing, parsing, and static semantics, but not exactly. For
// details, start with the comments at the top of parsing/pass1.rs, and follow the thread that it
// describes.

pub mod diagnostics;
pub mod model;
pub mod pass1;
pub mod pass2;
pub mod pass3;
pub mod score;
pub(crate) mod score_helpers;
mod timeline;
use crate::parsing::diagnostics::Diagnostics;
use crate::parsing::score::{Directive, FromRawDirective};
pub use timeline::*;

pub fn parse<'s>(input: &'s str) -> Result<Timeline<'s>, Diagnostics> {
    pass3::parse3(input)
}

pub fn show_help() -> anyhow::Result<()> {
    Ok(Directive::show_help(&mut anstream::stdout())?)
}

#[cfg(test)]
mod tests;
