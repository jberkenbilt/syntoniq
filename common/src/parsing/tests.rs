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
// For a quick way to check all spans, load the json and sqt files into emacs in two windows of the
// same frame, position the point at the beginning of the JSON file, and repeatedly run the emacs
// lisp function below. This highlights each span in turn.. This works equally well for errors and
// correctly tokenized files.

/*
```elisp
(defun highlight-next-span()
  "Highlight next span. Run with test output json in one window
and the source file in the next window."
  (interactive)
  (require 'hi-lock)
  (if (search-forward "\"span\": [" nil t)
      (progn
        ;; Find the beginning and end of the span start and end markers.
        ;; This parses JSON lexically by assuming that jumping forward and
        ;; backward by expressions will skip over the numbers. This should
        ;; be a safe assumption.
        (let* ((start-end (progn (forward-sexp) (point)))
               (end-end (progn (forward-sexp) (point)))
               (end-start (progn (backward-sexp) (point)))
               (start-start (progn (backward-sexp) (point)))
               ;; Read the numbers from the buffer
               (span-start (string-to-number (buffer-substring start-start start-end)))
               (span-end (string-to-number (buffer-substring end-start end-end)))
              )
          (save-excursion
            (other-window 1)
            (let* ((start (1+ (or (byte-to-position span-start) 0)))
                   (end (1+ (or (byte-to-position span-end) 0)))
                   (ov (make-overlay start end)))
              (remove-overlays (point-min) (point-max) 'syntoniq-next-span t)
              (overlay-put ov 'face 'hi-pink)
              (overlay-put ov 'syntoniq-next-span t)
            )
            (other-window -1)
          )
        )
      )
    (other-window 1)
    (remove-overlays (point-min) (point-max) 'syntoniq-next-span t)
    (other-window -1)
  )
)
```
*/

use crate::parsing::diagnostics::Diagnostics;
use crate::parsing::model::Spanned;
use crate::parsing::pass1::parse1;
use crate::parsing::pass2::parse2;
use serde::Serialize;
use serde_json::json;
use std::fmt::{Debug, Display};
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

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
            errors.push(format!("{}: FAIL", p.display()));
        }
    }
    if !errors.is_empty() {
        eprintln!("Run ./target/debug/tokenize file.stq --json to generate output for comparison");
        for e in errors {
            eprintln!("ERROR: {e}");
        }
        panic!("there were errors");
    }
    Ok(())
}
