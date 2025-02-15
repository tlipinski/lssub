use anyhow::Result;
use log::info;
use serde::Deserialize;
use std::{env, fs};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
}

#[derive(Debug, Deserialize)]
pub struct Api {
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
