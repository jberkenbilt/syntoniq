// Pass 3 of parsing is responsible for semantic checks. This includes validating specific
// directives and scale blocks and making sure that notes and dynamics are valid. After pass 3,
// there should be enough information to generate output or reformatted input.

// Many functions returns Option<()> o we can conveniently use the `?` operator on the return value
// of from_raw and anything else that returns an Option type. There's no need to use Result types
// here since errors are handled using the Diagnostics type.

use crate::parsing::diagnostics::{Diagnostics, code};
use crate::parsing::pass2::{Pass2, Token2};
use crate::parsing::score::{Directive, FromRawDirective, Score};
use crate::parsing::{Options, ScoreOutput, pass2};

fn check_init<'s>(
    src: &'s str,
    tokens: &[Token2],
    diags: &Diagnostics,
) -> Option<(usize, Score<'s>)> {
    for (i, tok) in tokens.iter().enumerate() {
        match &tok.value.t {
            Pass2::Space | Pass2::Newline | Pass2::Comment => continue,
            Pass2::Directive(raw) if raw.name.value == "syntoniq" => {
                if let Some(Directive::Syntoniq(x)) = Directive::from_raw(diags, tok.span, raw) {
                    return Some((i + 1, Score::new(src, x)));
                }
                break;
            }
            _ => break,
        }
    }
    diags.err(
        code::INITIALIZATION,
        0..1,
        "syntoniq file must start with syntoniq(version=n)",
    );
    None
}

pub fn parse3<'s>(src: &'s str, options: &Options) -> Result<ScoreOutput<'s>, Diagnostics> {
    let tokens = pass2::parse2(src)?;
    let diags = Diagnostics::new();
    let Some((skip, mut score)) = check_init(src, &tokens, &diags) else {
        return Err(diags);
    };

    let mut next_newline_is_blank_line = true;
    for tok in tokens.into_iter().skip(skip) {
        // Detect when we have to process a score block. Score blocks are groups of contiguous
        // score/dynamic lines, possibly intermixed with comments. They are terminated by any
        // other functional token, a line containing only white space, or EOF. EOF is handled
        // specially at the end of the loop.

        let terminates_score_block = match &tok.value.t {
            Pass2::Space | Pass2::Comment => false,
            Pass2::Newline => next_newline_is_blank_line,
            Pass2::Directive(_) => true,
            Pass2::NoteLine(_) | Pass2::DynamicLine(_) => false,
        };
        if terminates_score_block {
            score.handle_score_block(&diags);
        }

        // Score lines swallow up their whole line including comments and newlines. For a newline
        // to indicate a blank line, it must be seen after a score line or another newline without
        // any intervening non-space tokens. A newline at the beginning of the file is also a blank
        // line, though we don't actually care.
        next_newline_is_blank_line = match &tok.value.t {
            Pass2::Space => next_newline_is_blank_line,
            Pass2::Comment | Pass2::Directive(_) => false,
            Pass2::NoteLine(_) | Pass2::DynamicLine(_) | Pass2::Newline => true,
        };
        match tok.value.t {
            Pass2::Space | Pass2::Newline | Pass2::Comment => continue,
            Pass2::Directive(x) => score.handle_directive(&diags, tok.span, &x),
            Pass2::NoteLine(line) => score.add_note_line(line),
            Pass2::DynamicLine(line) => score.add_dynamic_line(line),
        };
    }
    score.handle_score_block(&diags);
    score.do_final_checks(&diags);
    score.post_process(&diags, options);
    if diags.has_errors() {
        Err(diags)
    } else {
        Ok(score.into_output())
    }
}
