// #[macro_use]
// extern crate log;
// extern crate log4rs;

use std::fs::{create_dir_all, exists};

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;

use colored::Colorize;
use sysinfo::System;

use chrono;

#[macro_export]
macro_rules! temp_dir {
    () => {
        std::env::temp_dir().join("picoman")
    };
}

pub fn nix_workaround() {
    if System::name().unwrap() == "NixOS" {
        println!(
            "{}{}",
            "Running on NixOS
If you can't launch the gui, run:"
                .red(),
            "
nix-shell --run \"just runnix\""
                .green()
        );
    }
}

pub fn logger_init() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = ConsoleAppender::builder().build();

    let localtime = chrono::Local::now().format("%d-%m-%Y_%H.%M.%S");

    let logs_dir = std::path::PathBuf::from("picoman_files").join("logs");

    if !exists(&logs_dir)? {
        create_dir_all(&logs_dir)?;
    } //create new dir if it doesn't exist already

    let path = logs_dir.join(format!("{localtime}.log"));

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(path)?;

    let config_stdout = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Info)))
                .build("stdout", Box::new(stdout)),
            //Appender for stdout has a filter for "info" log level
        )
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        //That's the same appender but without filters, so the file has trace log level
        .build(
            Root::builder()
                .appender("stdout")
                .appender("logfile")
                .build(LevelFilter::Trace),
        )?;

    log4rs::init_config(config_stdout)?;

    Ok(())
}
