#![allow(unused)]

mod cli;
mod config;
mod secret;
mod ui;

use crate::cli::gui_cmd::handle_gui_cmd;
use crate::config::get_config;
use crate::secret::retrieve;
use anyhow::{Error, Result};
use clap::Parser;
use env_logger::{Builder, Target};
use log::{LevelFilter, error, info, warn};
use osb::user_info::get_user_info;
use std::fs::OpenOptions;
use std::path::PathBuf;
use ui::app::App;

#[tokio::main]
async fn main() {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("/tmp/subster.log")
        .expect("Failed to open log file");

    // Configure env_logger to write logs to the file
    Builder::new()
        .target(Target::Pipe(Box::new(file)))
        .filter_level(LevelFilter::Debug)
        .init();

    info!("Starting");

    let args = Args::parse();

    info!("{args:?}");

    match run(args).await {
        Ok(_) => {}
        Err(e) => {
            // error!("{e:?}");
            error!("{e}");
        }
    };
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    name: Option<String>,
}

async fn run(args: Args) -> Result<()> {
    let _ = get_config()?;

    handle_gui_cmd(args.name.as_deref()).await
}
