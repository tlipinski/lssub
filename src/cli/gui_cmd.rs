use crate::ui::app::App;
use anyhow::Result;
use osb::subtitles::subtitles;
use std::path::Path;

pub async fn handle_gui_cmd(file_path: Option<&str>) -> Result<()> {
    let file_name = match file_path {
        None => "",

        Some(path) => Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?,
    };

    let mut terminal = ratatui::init();
    App::run(&mut terminal, file_name.into());
    ratatui::restore();

    Ok(())
}
