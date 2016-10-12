#[macro_use]
extern crate log;
extern crate env_logger;
extern crate clap;
extern crate toml;
extern crate rustc_serialize;
extern crate regex;
extern crate glob;

#[macro_use]
mod utils;  // first because of macros inside it
mod check;
mod config;

use config::Config;
use utils::unwrap_or_exit;

use log::LogLevelFilter;
use env_logger::LogBuilder;
use clap::{App, Arg, ArgMatches};

use std::process;

const DEFAULT_CONFIG: &'static str = ".cargo-tidy.toml";

fn init_logging(matches: &ArgMatches) {
    let default_level = if matches.is_present("DEBUG") {
        LogLevelFilter::Debug
    } else {
        LogLevelFilter::Info
    };

    LogBuilder::new()
        .filter(None, default_level)
        .init()
        .expect("logger initialization failed");

    debug!("logging initialized");
}

fn main() {
    let args = App::new("cargo-tidy")
        .subcommand(App::new("tidy")
            .arg(Arg::with_name("CONFIG_FILE")
                .long("config")
                .short("c")
                .takes_value(true)
                .help("Use the given config file"))
            .arg(Arg::with_name("DEBUG")
                .long("debug")
                .short("d")
                .help("Enable debug logging")))
        .get_matches();

    let args = unwrap_or_exit(args.subcommand_matches("tidy")
        .ok_or("cargo-tidy must be invoked as a cargo subcommand"));

    init_logging(&args);

    let config_file = args.value_of("CONFIG_FILE").unwrap_or(DEFAULT_CONFIG);
    let config = unwrap_or_exit(Config::load(config_file));

    debug!("loaded config: {:?}", config);

    match check::run_checks(&config) {
        Ok(()) => {
            println!("all tidy checks passed without error");
        }
        Err(errs) => {
            assert!(!errs.is_empty());

            errprintln!("{} tidy error{}", errs.len(), if errs.len() == 1 { "" } else { "s" });
            for err in &errs {
                errprintln!("{}", err);
            }

            process::exit(1);
        }
    }
}
