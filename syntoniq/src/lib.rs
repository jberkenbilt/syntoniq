use syntoniq_common::parsing;
use syntoniq_common::parsing::Timeline;
pub mod generator;

pub fn parse<'s>(filename: &str, src: &'s str) -> anyhow::Result<Timeline<'s>> {
    match parsing::parse(src) {
        Ok(timeline) => Ok(timeline),
        Err(diags) => {
            anstream::eprintln!("{}", diags.render(filename, src));
            std::process::exit(2);
        }
    }
}
