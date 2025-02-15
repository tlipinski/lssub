mod config;
mod login;
mod secret;
mod user_info;
mod values;
mod cli;

use crate::cli::command::Command;
use crate::config::get_config;
use crate::secret::{retrieve};
use crate::user_info::get_user_info;
use anyhow::{Error, Result};
use clap::Parser;
use log::{error, info};
use crate::cli::login_cmd::handle_login_cmd;
use crate::cli::logout_cmd::handle_logout_cmd;

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Starting");

    let args = Args::parse();

    match run(args).await {
        Ok(_) => {}
        Err(e) => {
            error!("{e}")
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
    let config = get_config()?;

    match args.command {
        Command::Login => {
            handle_login_cmd().await
        }
        Command::Logout => {
            handle_logout_cmd().await
        }
        Command::UserInfo => {
            if let Some(token) = retrieve().await? {
                get_user_info(&token).await
            } else {
                Err(Error::msg("Login first"))
            }
        }
    }

}
