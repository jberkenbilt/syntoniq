use anyhow::bail;
use std::{env, fs};
use syntoniq_common::parsing::pass2;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let Some(filename) = args.get(1) else {
        bail!("specify filename as argument");
    };
    let data = fs::read(filename)?;
    let input = str::from_utf8(&data)?;
    match pass2::parse2(input) {
        Err(diags) => {
            println!("{diags}");
        }
        Ok(tokens) => {
            for t in tokens {
                println!("{t}")
            }
        }
    }
    Ok(())
}
