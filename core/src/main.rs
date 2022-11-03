use clap::{arg, command, ArgAction};
use core::run;
fn main() {
    let matches = command!()
        .arg(arg!(-v - -verbose).action(ArgAction::Count))
        .get_matches();
    let log_level = matches
        .get_one::<u8>("verbose")
        .expect("Count always defaulted");
    let filter = match log_level {
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Warn,
        3 => log::LevelFilter::Info,
        4 => log::LevelFilter::Debug,
        5 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Off,
    };
    let mut builder = env_logger::builder();
    builder.filter_level(filter).init();

    pollster::block_on(run());
}
