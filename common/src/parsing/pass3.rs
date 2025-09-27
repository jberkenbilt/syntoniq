// Pass 3 of parsing is responsible for semantic checks. This includes validating specific
// directives and scale blocks and making sure that notes and dynamics are valid. After pass 3,
// there should be enough information to generate output or reformatted input.

// Many functions returns Option<()> o we can conveniently use the `?` operator on the return value
// of from_raw and anything else that returns an Option type. There's no need to use Result types
// here since errors are handled using the Diagnostics type.

use crate::parsing::diagnostics::{Diagnostics, code};
use crate::parsing::model::RawDirective;
use crate::parsing::pass2;
use crate::parsing::pass2::Pass2;
use crate::parsing::score::{Directive, Score};
use crate::parsing::score_helpers::FromRawDirective;

fn pre_init(diags: &Diagnostics, d: &RawDirective) -> Option<Score> {
    if let Directive::Syntoniq(x) = Directive::from_raw(diags, d)? {
        Some(Score::new(x))
    } else {
        None
    }
}

pub fn parse3<'s>(src: &'s str) -> Result<Score, Diagnostics> {
    let tokens = pass2::parse2(src)?;
    let diags = Diagnostics::new();
    let mut option_score: Option<Score> = None;

    let mut non_space_since_last_newline = false;

    for tok in tokens {
        match tok.value.t {
            Pass2::Newline => {
                if non_space_since_last_newline {
                    non_space_since_last_newline = false;
                } else if let Some(s) = option_score.as_mut() {
                    s.handle_score_block(&diags);
                }
                continue;
            }
            Pass2::Space | Pass2::Comment => continue,
            _ => {}
        }
        non_space_since_last_newline = true;
        let Some(score) = option_score.as_mut() else {
            option_score = match tok.value.t {
                Pass2::Directive(x) => pre_init(&diags, &x),
                _ => None,
            };
            if option_score.is_none() {
                diags.err(code::INITIALIZATION, tok.span, "syntonic is not initialized -- the first directive must be syntoniq(version=n)");
                return Err(diags);
            }
            continue;
        };

        // pending_scale will be `Some` when the last operation was a scale definition.
        let mut pending_scale = score.pending_scale.take();
        match tok.value.t {
            Pass2::Space | Pass2::Newline | Pass2::Comment => unreachable!(),
            Pass2::Directive(x) => score.handle_directive(&diags, &x),
            Pass2::NoteLine(line) => score.add_note_line(line),
            Pass2::DynamicLine(line) => score.add_dynamic_line(line),
            Pass2::ScaleBlock(x) => score.handle_scale_block(&diags, pending_scale.take(), &x),
        };
        if pending_scale.is_some() {
            diags.err(
                code::USAGE,
                tok.span,
                "a scale block immediately follow a scale definition",
            );
        }
    }
    match option_score {
        None => diags.err(code::INITIALIZATION, 0..1, "Syntoniq was never initialized"),
        Some(mut score) => {
            score.handle_score_block(&diags);
            if !diags.has_errors() {
                return Ok(score);
            }
        }
    }
    Err(diags)
}
