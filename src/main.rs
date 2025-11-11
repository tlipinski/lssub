#![allow(unused)]

mod cli;
mod config;
mod secret;
mod ui;

use crate::cli::command::Command;
use crate::cli::features_cmd::handle_features_cmd;
use crate::cli::gui_cmd::handle_gui_cmd;
use crate::cli::login_cmd::handle_login_cmd;
use crate::cli::logout_cmd::handle_logout_cmd;
use crate::cli::search_cmd::handle_search_cmd;
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
    #[command(subcommand)]
    command: Option<Command>,
}

async fn run(args: Args) -> Result<()> {
    let _ = get_config()?;

    match args.command {
        Some(Command::Login) => handle_login_cmd().await,

        Some(Command::Logout) => {
            if retrieve().await?.is_some() {
                handle_logout_cmd().await
            } else {
                Err(Error::msg("Already logged out"))
            }
        }

        Some(Command::UserInfo) => {
            if let Some(token) = retrieve().await? {
                get_user_info(&token).await
            } else {
                Err(Error::msg("Login first"))
            }
        }

        Some(Command::Search {
            file_path,
            languages,
        }) => handle_search_cmd(&file_path, languages).await,

        Some(Command::Features { query }) => handle_features_cmd(&query).await,

        None => handle_gui_cmd(args.name.as_deref()).await,
    }
}
