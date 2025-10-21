use crate::generator;
use crate::generator::GenerateOptions;
use std::fs;
use std::path::{Path, PathBuf};
use syntoniq_common::parsing::Options;
use syntoniq_common::test_helpers;

#[test]
fn test_generator() -> anyhow::Result<()> {
    // This is designed to fail if anything failed but to run all the tests and produce useful
    // output for analysis. For each file, we generate a MIDI file and a csound file and compare
    // literally. If they are different, we save the generated file for comparison.
    //
    // Every test file contains comments explaining what the test is doing and what to listen for.
    //
    // A difference may indicate a test failure or an intentional change. To validate, you can do
    // any or all of the following:
    // - Diff the JSON files
    // - Use `midicsv` to generate CSV for the MIDI files and diff
    // - Diff the csound files
    // - Listen to the files, tracking with the comments in the source
    let paths = test_helpers::get_stq_files("test-data")?;
    let mut errors = Vec::<String>::new();
    // Create a temporary directory for testing.
    let tmp = tempfile::tempdir()?;
    let temp_dir = Path::join(tmp.path(), "stq");
    fs::create_dir_all(&temp_dir)?;

    for p in paths {
        let base = p.file_name().unwrap().to_string_lossy().replace(".stq", "");
        let outfile = |suf: &str| Path::join(&temp_dir, format!("{base}.{suf}"));
        let input_file = |suf: &str| format!("test-data/{base}.{suf}");
        let savefile = |suf: &str| format!("test-data/actual/{base}.{suf}");
        let csound_template = Some(input_file("template.csd")).and_then(|x| {
            let p = PathBuf::from(x);
            if fs::exists(&p).unwrap() {
                Some(p)
            } else {
                None
            }
        });
        let options = GenerateOptions {
            score: input_file("stq").into(),
            json: Some(outfile("json")),
            midi: Some(outfile("midi")),
            csound: Some(outfile("csd")),
            csound_template,
            parse_options: Default::default(),
        };
        if let Err(e) = generator::run(options) {
            errors.push(format!("{base}: {e}"));
        }
        for suf in ["json", "midi", "csd"] {
            let actual = fs::read(outfile(suf))?;
            let exp = fs::read(input_file(suf)).unwrap_or_default();
            if actual == exp {
                println!("{base}: {suf} PASSED");
            } else {
                let save = savefile(suf);
                fs::create_dir_all(PathBuf::from(&save).parent().unwrap())?;
                fs::write(save, actual)?;
                errors.push(format!("{base}: {suf} FAILED; saved in actual"));
            }
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

#[test]
fn test_options() -> anyhow::Result<()> {
    // See comments in test_generator for the general flow.
    let mut errors = Vec::<String>::new();
    let tmp = tempfile::tempdir()?;
    let temp_dir = Path::join(tmp.path(), "stq");
    fs::create_dir_all(&temp_dir)?;
    let test_cases = [
        (
            "explicit-default",
            Options {
                start_mark: None,
                end_mark: None,
                skip_repeats: false,
                tempo_percent: Some(100),
            },
        ),
        (
            "only-tempo-percent",
            Options {
                start_mark: None,
                end_mark: None,
                skip_repeats: false,
                tempo_percent: Some(200),
            },
        ),
        (
            "tempo-no-repeats",
            Options {
                start_mark: None,
                end_mark: None,
                skip_repeats: true,
                tempo_percent: Some(150),
            },
        ),
        (
            "start-end-tempo-no-repeats",
            Options {
                start_mark: Some("chorus-main-start".to_string()),
                end_mark: Some("verse-end".to_string()),
                skip_repeats: true,
                tempo_percent: Some(75),
            },
        ),
        (
            "start-end",
            Options {
                start_mark: Some("chorus-main-start".to_string()),
                end_mark: Some("verse-end".to_string()),
                skip_repeats: false,
                tempo_percent: None,
            },
        ),
        (
            "start-only",
            Options {
                start_mark: Some("chorus-main-start".to_string()),
                end_mark: None,
                skip_repeats: false,
                tempo_percent: None,
            },
        ),
        (
            "end-only",
            Options {
                start_mark: None,
                end_mark: Some("verse-end".to_string()),
                skip_repeats: false,
                tempo_percent: None,
            },
        ),
        (
            "later-start",
            Options {
                start_mark: Some("verse-end".to_string()),
                end_mark: None,
                skip_repeats: false,
                tempo_percent: None,
            },
        ),
        (
            "end-near-start",
            Options {
                start_mark: None,
                end_mark: Some("verse-start".to_string()),
                skip_repeats: false,
                tempo_percent: None,
            },
        ),
        (
            "start-near-end",
            Options {
                start_mark: Some("ending".to_string()),
                end_mark: None,
                skip_repeats: false,
                tempo_percent: None,
            },
        ),
    ];

    for (name, parse_options) in test_cases {
        let base = "test12-nested-repeats";
        let outfile = |suf: &str| Path::join(&temp_dir, format!("{base}.{name}.{suf}"));
        let orig_file = |suf: &str| format!("test-data/{base}.{suf}");
        let exp_file = |suf: &str| format!("test-data/{base}.{name}.{suf}");
        let savefile = |suf: &str| format!("test-data/actual/{base}.{name}.{suf}");
        let options = GenerateOptions {
            score: orig_file("stq").into(),
            json: Some(outfile("json")),
            midi: Some(outfile("midi")),
            csound: Some(outfile("csd")),
            csound_template: None,
            parse_options,
        };
        if let Err(e) = generator::run(options) {
            errors.push(format!("{base}: {e}"));
        }
        for suf in ["json", "midi", "csd"] {
            let actual = fs::read(outfile(suf))?;
            let exp = fs::read(exp_file(suf)).unwrap_or_default();
            if actual == exp {
                println!("{base}: {name}, {suf} PASSED");
            } else {
                let save = savefile(suf);
                fs::create_dir_all(PathBuf::from(&save).parent().unwrap())?;
                fs::write(save, actual)?;
                errors.push(format!("{base}: {name}, {suf} FAILED; saved in actual"));
            }
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
