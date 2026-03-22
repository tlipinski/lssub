use crate::values::APP_NAME;
use anyhow::Result;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use xdg::BaseDirectories;

#[derive(Debug)]
pub struct ConfigProvider {
    prefix: String,
    path: PathBuf,
}

impl ConfigProvider {
    fn xdg_dirs(&self) -> BaseDirectories {
        BaseDirectories::with_prefix(self.prefix.clone())
    }

    fn config_path(&self) -> PathBuf {
        self.xdg_dirs()
            .get_config_file(self.path.clone())
            .expect("HOME does not exist")
    }

    pub fn modify(&self, f: impl Fn(&Config) -> Config) -> Result<()> {
        let c = self.get_config()?;
        let updated = f(&c);
        self.save_config(&updated)
    }

    pub fn get_config(&self) -> Result<Config> {
        info!("Loading config from: {}", self.config_path().display());
        if self.config_path().exists() {
            let contents = match fs::read_to_string(self.config_path()) {
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
            self.xdg_dirs().place_config_file(&self.path)?;
            self.save_config(&default)?;
            Ok(default)
        }
    }

    pub fn save_config(&self, config: &Config) -> Result<()> {
        fs::write(self.config_path(), toml::to_string(&config)?)?;
        Ok(())
    }
}

impl Default for ConfigProvider {
    fn default() -> Self {
        ConfigProvider {
            prefix: APP_NAME.to_string(),
            path: "config.toml".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
