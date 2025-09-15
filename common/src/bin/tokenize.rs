use anyhow::bail;
use std::{env, fs};
use syntoniq_common::parsing::lexer;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let Some(filename) = args.get(1) else {
        bail!("specify filename as argument");
    };
    let data = fs::read(filename)?;
    let input = str::from_utf8(&data)?;
    match lexer::lex(input) {
        Err(diags) => {
            println!("{diags}");
        }
        Ok(tokens) => {
            for t in tokens {
                println!("{t:?}")
            }
        }
    }
    Ok(())
}
