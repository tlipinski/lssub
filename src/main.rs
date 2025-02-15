mod cli;
mod config;
mod login;
mod login_cmd;
mod secret;
mod user_info;
mod values;
mod logout_cmd;

use crate::cli::Command;
use crate::config::get_config;
use crate::login::{login, Credentials};
use crate::login_cmd::handle_login_cmd;
use crate::secret::{retrieve, store};
use crate::user_info::get_user_info;
use crate::values::{xor, API_URL, KEY, USER_AGENT};
use anyhow::{Error, Result};
use clap::Parser;
use hex_literal::hex;
use log::{error, info};
use secrecy::ExposeSecret;
use std::io::{stdin, stdout, Write};
use crate::logout_cmd::handle_logout_cmd;

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
    command: cli::Command,
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
                get_user_info(&config, &token).await
            } else {
                Err(Error::msg("Login first"))
            }
        }
    }

}
