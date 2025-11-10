use anyhow::Result;
use log::{debug, info};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {}

#[derive(Debug, Deserialize)]
pub struct Api {}

pub fn get_config() -> Result<Config> {
    info!("Loading config");
    let xdg_dirs = xdg::BaseDirectories::with_prefix("subster")?;
    debug!("XDG: {:?}", xdg_dirs);
    let config_path = xdg_dirs.get_config_file("config.toml");
    let contents = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&contents)?;
    info!("Config loaded {:?}", config);
    Ok(config)
}
