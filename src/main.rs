mod config;
mod login;
mod user_info;

use crate::config::get_config;
use crate::login::login;
use crate::user_info::get_user_info;
use anyhow::Result;
use log::{error, info};

#[tokio::main]
async fn main() {
    env_logger::init();

    match run().await {
        Ok(_) => {}
        Err(e) => {
            error!("Error: {e}")
        }
    };
}

async fn run() -> Result<()> {
    let config = get_config()?;

    let user = std::env::var("USER")?;
    let pass = std::env::var("PASS")?;

    println!("{}", user);
    println!("{}", pass);


    let t = login(&config, &user, &pass).await?;
    let r = get_user_info(&config).await?;

    Ok(())
}
