use anyhow::Result;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub languages: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            languages: vec!["en".into()],
        }
    }
}

pub fn get_config() -> Result<Config> {
    info!("Loading config from: {:?}", config_path());
    if config_path()?.exists() {
        let contents = match fs::read_to_string(config_path()?) {
            Ok(raw) => raw,
            Err(e) => {
                error!("Failed to read config file: {}", e);
                std::process::exit(1);
            }
        };
        let config: Config = toml::from_str(&contents)?;
        info!("Config loaded: {config:?}");
        Ok(config)
    } else {
        let default = Config::default();
        save_config(&default)?;
        Ok(default)
    }
}

pub fn save_config(config: &Config) -> Result<()> {
    fs::write(config_path()?, toml::to_string(&config)?);
    Ok(())
}

fn config_path() -> Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("subster")?;
    Ok(xdg_dirs.get_config_file("config.toml"))
}