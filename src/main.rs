use clap::{Arg, ArgAction, Command, value_parser};
use clap_complete::{Generator, Shell, aot};
use qlaunchpad::controller::Controller;
use std::error::Error;
use std::io;

fn build_cli() -> Command {
    //TODO: decide if I want to use derive or arg macro, get completion working, figure out real
    // syntax
    Command::new("example")
        .arg(
            Arg::new("port")
                .long("port")
                .help("midi port name (amidiplay -l)"),
        )
        .arg(
            Arg::new("no-prog")
                .long("no-prog")
                .help("don't enter programmer mode")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("generator")
                .long("generate")
                .action(ArgAction::Set)
                .value_parser(value_parser!(Shell)),
        )
}

fn print_completions<G: Generator>(generator: G, cmd: &mut Command) {
    aot::generate(
        generator,
        cmd,
        cmd.get_name().to_string(),
        &mut io::stdout(),
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    ctrlc::set_handler(move || {
        println!("TODO: restore live mode; hit enter to exit");
    })?;
    let matches = build_cli().get_matches();

    if let Some(generator) = matches.get_one::<Shell>("generator").copied() {
        let mut cmd = build_cli();
        eprintln!("Generating completion file for {generator}...");
        print_completions(generator, &mut cmd);
    }
    if let Some(port) = matches.get_one::<String>("port") {
        let no_prog = matches
            .get_one::<bool>("no-prog")
            .copied()
            .unwrap_or_default();
        Controller::run(port, no_prog)?;
    }
    Ok(())
}
