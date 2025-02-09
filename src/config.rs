use anyhow::Result;
use log::info;
use secrecy::SecretBox;
use serde::Deserialize;
use std::{env, fs};
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub api: Api,
}

#[derive(Debug, Deserialize)]
pub struct Api {
    pub url: String,
    pub key: SecretBox<String>,
    pub token: SecretBox<String>,
}

pub fn get_config() -> Result<Config> {
    info!("Loading config");
    let home = env::var("HOME")?;
    let config_path = PathBuf::from(home).join(".config/subster/config.toml");
    let contents = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&contents)?;
    info!("Config loaded {:?}", config);
    Ok(config)
}
