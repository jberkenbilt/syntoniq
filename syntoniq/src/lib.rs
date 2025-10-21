use syntoniq_common::parsing;
use syntoniq_common::parsing::Timeline;
pub mod generator;
pub use parsing::Options;

pub fn parse<'s>(filename: &str, src: &'s str, options: &Options) -> anyhow::Result<Timeline<'s>> {
    match parsing::parse(src, options) {
        Ok(timeline) => Ok(timeline),
        Err(diags) => {
            anstream::eprintln!("{}", diags.render(filename, src));
            std::process::exit(2);
        }
    }
}
