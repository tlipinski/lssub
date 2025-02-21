#![allow(unused)]

mod cli;
mod config;
mod secret;
mod app;

use crate::cli::command::Command;
use crate::cli::login_cmd::handle_login_cmd;
use crate::cli::logout_cmd::handle_logout_cmd;
use crate::cli::search_cmd::handle_search_cmd;
use crate::config::get_config;
use crate::secret::retrieve;
use anyhow::{Error, Result};
use clap::Parser;
use log::{error, info};
use osb::user_info::get_user_info;
use crate::app::App;

#[tokio::main]
async fn main() {
    env_logger::init();

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
    #[command(subcommand)]
    command: Command,
}

async fn run(args: Args) -> Result<()> {
    let _ = get_config()?;

    match args.command {
        Command::Login => handle_login_cmd().await,

        Command::Logout => {
            if let Some(_) = retrieve().await? {
                handle_logout_cmd().await
            } else {
                Err(Error::msg("Already logged out"))
            }
        }

        Command::UserInfo => {
            if let Some(token) = retrieve().await? {
                get_user_info(&token).await
            } else {
                Err(Error::msg("Login first"))
            }
        }

        Command::Search {
            file_path,
            languages,
        } => {
            handle_search_cmd(&file_path, languages).await?;
            Ok(())
        }

        Command::Gui => {
            let mut terminal = ratatui::init();
            let _ = App::default().run(&mut terminal);
            ratatui::restore();
            Ok(())
        }
    }
}
