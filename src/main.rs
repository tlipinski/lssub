mod cli;
mod config;
mod login;
mod secret;
mod user_info;
mod values;

use crate::cli::Command;
use crate::config::get_config;
use crate::login::{login, Credentials};
use crate::secret::{retrieve, store};
use crate::user_info::get_user_info;
use anyhow::Result;
use clap::Parser;
use log::{error, info};
use std::io::{stdin, stdout, Write};
use hex_literal::hex;
use secrecy::ExposeSecret;
use crate::values::{xor, API_URL, KEY, USER_AGENT};

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
            let mut username = String::new();
            let mut password = String::new();

            print!("Username: ");
            stdout().flush()?;
            stdin().read_line(&mut username)?;

            print!("Password: ");
            stdout().flush()?;
            stdin().read_line(&mut password)?;

            username = username.trim().to_string();
            password = password.trim().to_string();

            let credentials = Credentials {
                username: username.clone(),
                password: password,
            };

            let api_token = login(&credentials).await?;

            let _ = store(&api_token, &username).await?;

            println!("Logged in successfully")
        }
    }

    let api_token = if let Some(token) = retrieve().await? {
        token
    } else {
        let username = std::env::var("USER")?;
        let password = std::env::var("PASS")?;

        let credentials = Credentials {
            username: username.clone(),
            password,
        };

        let api_token = login(&credentials).await?;

        let _ = store(&api_token, &username).await?;

        api_token
    };

    println!("{}", api_token.0.expose_secret());

    let _ = get_user_info(&config, &api_token).await?;

    Ok(())
}
