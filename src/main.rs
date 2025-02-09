use anyhow::Result;
use log::{error, info};
use secrecy::SecretBox;
use serde::Deserialize;
use std::env;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    env_logger::init();

    match get_config() {
        Ok(conf) => {
            info!("{:?}", conf.api.key);
        }
        Err(e) => {
            error!("{:?}", e);
        }
    };
}

#[derive(Debug, Deserialize)]
pub struct Config {
    api: Api,
}

#[derive(Debug, Deserialize)]
struct Api {
    key: SecretBox<String>,
}

fn get_config() -> Result<Config> {
    info!("Loading config");
    let home = env::var("HOME")?;
    let config_path = PathBuf::from(home).join(".config/subster/config.toml");
    let mut opened = OpenOptions::new().read(true).open(config_path)?;
    let mut contents = String::new();
    opened.read_to_string(&mut contents)?;
    let c: Config = toml::from_str(&contents)?;
    info!("Config loaded {:?}", c);
    Ok(c)
}
