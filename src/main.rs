use clap::{Arg, ArgAction, Command, value_parser};
use clap_complete::{Generator, Shell, aot};
use log::LevelFilter;
use qlaunchpad::controller::{Controller, ToDevice};
use std::error::Error;
use std::{env, io, thread};

fn build_cli() -> Command {
    //TODO: decide if I want to use derive or arg macro, get completion working, figure out real
    // syntax
    Command::new("example")
        .arg(
            Arg::new("port")
                .long("port")
                .required(true)
                .help("midi port name (amidiplay -l)"),
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

fn to_sync_send(e: Box<dyn Error>) -> Box<dyn Error + Sync + Send> {
    e.to_string().into()
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let mut log_builder = env_logger::builder();
    if env::var("RUST_LOG").is_err() {
        println!("Logging is controlled with RUST_LOG; see docs for the env_logger crate.");
        println!("Defaulting to INFO.");
        log_builder.filter_level(LevelFilter::Info);
    }
    log_builder.init();
    let matches = build_cli().get_matches();
    if let Some(generator) = matches.get_one::<Shell>("generator").copied() {
        let mut cmd = build_cli();
        eprintln!("Generating completion file for {generator}...");
        print_completions(generator, &mut cmd);
        return Ok(());
    }
    let port = matches.get_one::<String>("port").unwrap();
    let mut c = Controller::new(port).map_err(to_sync_send)?;
    let sender = c.sender();
    ctrlc::set_handler(move || {
        let _ = sender.send(ToDevice::Shutdown);
    })?;
    let sender = c.sender();
    let th = thread::spawn(move || c.run().map_err(to_sync_send));
    sender.send(ToDevice::Data(vec![0x90, 59, 0x2d]))?;
    log::info!("Hit CTRL-C to exit");
    th.join().unwrap()
}
