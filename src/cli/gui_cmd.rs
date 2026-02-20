use crate::ui::app::App;
use anyhow::Result;
use log::{error, info, warn};
use osb::subtitles::subtitles;
use std::path::{Path, PathBuf};
use std::process::exit;

pub async fn handle_gui_cmd(path_opt: Option<&str>) -> Result<()> {
    let p = if let Some(path) = path_opt {
        let canon_res = PathBuf::from(&path).canonicalize();

        match canon_res {
            Ok(canon) => {
                if (canon.is_absolute()) {
                    canon
                } else {
                    let current_dir = std::env::current_dir()?;
                    info!("cwd: {}", current_dir.display());

                    current_dir.join(&path)
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
        PathBuf::from(current_dir)
    };

    info!("Input path: {:?}", p);

    let (base_path, file_name) = if (p.is_dir()) {
        (Some(p.as_path()), None)
    } else {
        (p.parent(), p.file_stem().and_then(|os_str| os_str.to_str()))
    };

    if let Some(bp) = base_path{
        let mut terminal = ratatui::init();
        info!("Base path: {:?}", bp);
        info!("File name: {:?}", file_name);
        App::run(&mut terminal, bp, file_name);
        ratatui::restore();

        Ok(())
    } else {
        error!("Invalid path: {:?}", base_path);
        exit(1)
    }

}
