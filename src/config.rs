use anyhow::Result;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ConfigProvider {
    prefix: String,
    path: PathBuf,
}

impl ConfigProvider {
    fn config_path(&self) -> Result<PathBuf> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix(self.prefix.clone())?;
        Ok(xdg_dirs.get_config_file(self.path.clone()))
    }

    pub fn modify(&self, mut f: impl FnMut(&mut Config) -> ()) -> Result<()> {
        let mut c = self.get_config()?;
        f(&mut c);
        self.save_config(&c)
    }

    pub fn get_config(&self) -> Result<Config> {
        info!("Loading config from: {:?}", self.config_path());
        if self.config_path()?.exists() {
            let contents = match fs::read_to_string(self.config_path()?) {
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
            self.save_config(&default)?;
            Ok(default)
        }
    }

    pub fn save_config(&self, config: &Config) -> Result<()> {
        fs::write(self.config_path()?, toml::to_string(&config)?);
        Ok(())
    }
}

impl Default for ConfigProvider {
    fn default() -> Self {
        ConfigProvider {
            prefix: "subster".to_string(),
            path: "config.toml".into(),
        }
    }
}

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
