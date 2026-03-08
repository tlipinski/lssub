#![allow(unused)]

mod config;
mod osb;
mod secret;
mod ui;

use crate::config::{Config, ConfigProvider};
use crate::osb::user_info::get_user_info;
use crate::secret::retrieve;
use anyhow::{Error, Result};
use clap::Parser;
use env_logger::{Builder, Target};
use log::{LevelFilter, error, info, warn};
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::exit;
use ui::app::App;
use crate::osb::values::APP_NAME;

#[tokio::main]
async fn main() {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(format!("/tmp/{APP_NAME}.log"))
        .expect("Failed to open log file");

    // Configure env_logger to write logs to the file
    Builder::new()
        .target(Target::Pipe(Box::new(file)))
        .filter_level(LevelFilter::Debug)
        .init();

    info!("Starting");

    let args = Args::parse();

    info!("{args:?}");

    match run(args).await {
        Ok(_) => {}
        Err(e) => {
            error!("{e}");
        }
    };
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    path: Option<String>,
}

async fn run(args: Args) -> Result<()> {
    let path_opt = args.path.as_deref();
    let p = if let Some(path) = path_opt {
        let canon_res = PathBuf::from(&path).canonicalize();

        match canon_res {
            Ok(canon) => {
                if canon.is_absolute() {
                    canon
                } else {
                    let current_dir = std::env::current_dir()?;
                    info!("cwd: {}", current_dir.display());

                    current_dir.join(path)
                }
            }
            Err(err) => {
                warn!("{err}");
                let current_dir = std::env::current_dir()?;
                info!("cwd: {}", current_dir.display());
                current_dir
            }
        }
    } else {
        let current_dir = std::env::current_dir()?;
        info!("cwd: {}", current_dir.display());
        current_dir
    };

    info!("Input path: {:?}", p);

    let (base_path, file_name) = if (p.is_dir()) {
        (Some(p.as_path()), None)
    } else {
        (p.parent(), p.file_stem().and_then(|os_str| os_str.to_str()))
    };

    if let Some(bp) = base_path {
        let mut terminal = ratatui::init();
        info!("Base path: {:?}", bp);
        info!("File name: {:?}", file_name);
        App::run(&mut terminal, bp, file_name).await;
        ratatui::restore();

        Ok(())
    } else {
        error!("Invalid path: {:?}", base_path);
        exit(1)
    }
}
