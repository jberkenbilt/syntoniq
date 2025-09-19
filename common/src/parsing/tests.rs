// GENERAL STRATEGY
// - Create a file in the test directory whose name ends with stq
// - Use the tokenize program to tokenize it and inspect the regular output as prettified
//   tokens or error messages to ensure that everything is right
// - Use the tokenize program again to create JSON and store as file.json
// - Hand-check any spans or other details that are not otherwise exercised.
//
// Earlier tests do extensive validation of spans and so forth and were all hand-coded.
// Fully hand-coding this output would be extremely cumbersome and error-prone. At the point
// where these were automatically generated, there was high confidence of correctness. By having
// these, we can ensure that we don't regress on the current state or any future fixes.
//
// For a quick way to check all spans, load the json and sqt files into emacs and use a keyboard
// macro to search for the span in the json file and highlight the selected region in the stq
// file. My custom elisp functions highlight-region-by-offset and clear-region-highlights make
// this easy. With this approach, you can just repeat the keyboard macro and see every span in
// the JSON file to make sure all the spans are correct. This works equally well for errors
// and correctly tokenized files.

use crate::parsing::diagnostics::Diagnostics;
use crate::parsing::model::Spanned;
use crate::parsing::pass1::parse1;
use crate::parsing::pass2::parse2;
use serde::Serialize;
use std::fmt::{Debug, Display};
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use serde_json::json;

fn get_stq_files(dir: impl AsRef<Path>) -> anyhow::Result<Vec<PathBuf>> {
    let paths = fs::read_dir(dir)?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.display().to_string().ends_with(".stq") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    Ok(paths)
}

fn check_spans<T: Debug + Serialize>(exp_end: usize, tokens: &[Spanned<T>]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut exp_start: usize = 0;
    for (i, t) in tokens.iter().enumerate() {
        if t.span.start != exp_start {
            errors.push(format!(
                "token {i}: start={}; expected {exp_start}",
                t.span.start
            ));
        }
        exp_start = t.span.end;
    }
    if exp_start != exp_end {
        errors.push(format!(
            "last token ended with {exp_start}; expected {exp_end}"
        ))
    }
    errors
}

fn check_output<T: Debug + Serialize>(
    path: impl Display,
    in_len: usize,
    which: &str,
    errors: &mut Vec<String>,
    r: &Result<Vec<Spanned<T>>, Diagnostics>,
) {
    if let Ok(tokens) = r.as_ref() {
        for e in check_spans(in_len, tokens) {
            errors.push(format!("{path} check_spans for {which}: {e}"));
        }
    }
}

#[test]
fn test_pass2() -> anyhow::Result<()> {
    // This is designed to fail if anything failed but to run all the tests and produce useful
    // output for analysis.
    let paths = get_stq_files("parsing-tests/pass2")?;
    let mut errors = Vec::<String>::new();
    for p in paths {
        let in_data = String::from_utf8(fs::read(&p)?)?;
        let mut results: Vec<serde_json::Value> = Vec::new();
        let in_len = in_data.len();
        let path = p.display();
        let exp = p.to_str().unwrap().replace(".stq", ".json");
        let exp_value: serde_json::Value = serde_json::from_reader(File::open(&exp)?)?;

        // Pass 1
        let is_ok = {
            let r = parse1(&in_data);
            check_output(&path, in_len, "pass 1", &mut errors, &r);
            results.push(json!(&r));
            r.is_ok()
        };
        if is_ok {
            // Pass 2
            let r = parse2(&in_data);
            check_output(&path, in_len, "pass 2", &mut errors, &r);
            results.push(json!(&r));
        }

        let actual_value = json!(results);
        if actual_value == exp_value {
            eprintln!("{}: PASS", p.display());
        } else {
            // Generate output with ./target/debug/tokenize --json. Use eprintln on strings
            // rather than serde_json::to_writer_pretty to ensure we don't have interleaving.
            eprintln!("------ {} ------", p.display());
            eprintln!("------ ACTUAL ------");
            eprintln!("{}", serde_json::to_string_pretty(&actual_value)?);
            eprintln!("------ EXPECTED ------");
            eprintln!("{}", serde_json::to_string_pretty(&exp_value)?);
            eprintln!("------ END {} ------", p.display());
            errors.push(format!("{}: FAIL", p.display()));
        }
    }
    if !errors.is_empty() {
        for e in errors {
            eprintln!("ERROR: {e}");
        }
        panic!("there were errors");
    }
    Ok(())
}
