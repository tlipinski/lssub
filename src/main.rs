use anyhow::{Error, Result};
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;
use std::{env, io};
use toml::Value;

fn main() {
    match get_config() {
        Ok(conf) => {
            println!(
                "{:?}",
                conf.0
                    .as_table()
                    .unwrap()
                    .get("api")
                    .unwrap()
                    .get("key")
            );
        }
        Err(e) => {
            eprintln!("{:?}", e);
        }
    };
}

#[derive(Debug)]
pub struct Config(Value);

fn get_config() -> Result<Config> {
    let home = env::var("HOME")?;
    let config_path = PathBuf::from(home).join(".config/subster/config.toml");
    let mut opened = OpenOptions::new().read(true).open(config_path)?;
    let mut contents = String::new();
    opened.read_to_string(&mut contents)?;
    let parsed: Value = contents.parse()?;
    Ok(Config(parsed))
}
