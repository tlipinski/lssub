use anyhow::{Error, Result};
use log::info;
use secrecy::SecretBox;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;
use std::{env, io};
use secrecy::zeroize::DefaultIsZeroes;
use toml::Value;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    match get_config() {
        Ok(conf) => {
            println!(
                "{:?}",
                conf
            );
        }
        Err(e) => {
            eprintln!("{:?}", e);
        }
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    api: Api,
}

#[derive(Debug, Serialize, Deserialize)]
struct Api {
    key: String
}

fn get_config() -> Result<Config> {
    info!("Loading config");
    let home = env::var("HOME")?;
    let config_path = PathBuf::from(home).join(".config/subster/config.toml");
    let mut opened = OpenOptions::new().read(true).open(config_path)?;
    let mut contents = String::new();
    opened.read_to_string(&mut contents)?;

    let c: Config = toml::from_str(&contents)?;

    println!("{:?}", c);
    Ok(c)
}
