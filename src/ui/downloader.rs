use crate::osb::download::download;
use crate::osb::get_download_link::get_download_link;
use crate::secret::retrieve;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{ChangeStatus, DownloadedSubs};
use anyhow::{Error, Result};
use log::{debug, error, info};
use secrecy::ExposeSecret;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Clone)]
pub struct Downloader {
    base_path: PathBuf,
    file_name_opt: Option<String>,
    ui_tx: Sender<Action>,
}

impl Downloader {
    pub fn new(base_path: PathBuf, file_name_opt: Option<String>, ui_tx: Sender<Action>) -> Self {
        Downloader {
            base_path,
            file_name_opt,
            ui_tx,
        }
    }
    pub async fn download(&self, file_id: i64, language: &str) -> () {
        match self._download(file_id, language).await {
            Ok(action) => {
                self.ui_tx.send(action).await;
            }
            Err(err) => {
                self.ui_tx.send(ChangeStatus(err.to_string())).await;
            }
        }
    }

    async fn _download(&self, file_id: i64, language: &str) -> Result<Action> {
        info!("Downloading subs file: {file_id:?}");

        let token_opt = retrieve().await?;

        let download_link_response = get_download_link(token_opt, file_id).await.map_err(|e| {
            error!("Downloading subs failed: {e}");
            e
        })?;

        debug!("Download link response: {:?}", download_link_response);
        debug!("Base path: {:?}", self.base_path);
        debug!("File name: {:?}", self.file_name_opt);

        let content = download(download_link_response.link).await.map_err(|e| {
            error!("Downloading subs failed: {e}");
            e
        })?;

        let output_file = output_file(
            &self.base_path,
            &self.file_name_opt,
            download_link_response.file_name.as_str(),
            language,
        );

        debug!("Output file: {:?}", output_file);

        tokio::fs::write(output_file.clone(), content)
            .await
            .map_err(|e| {
                Error::msg(format!(
                    "Error saving subtitle file {}: {}",
                    output_file.display(),
                    e
                ))
            })?;

        Ok(DownloadedSubs(Downloaded {
            path: output_file,
            requests: download_link_response.requests,
            remaining: download_link_response.remaining,
        }))
    }
}

fn output_file(
    base_path: &Path,
    file_name_opt: &Option<String>,
    default_file_name: &str,
    language: &str,
) -> PathBuf {
    let default_path = Path::new(default_file_name);
    let default_stem = default_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| default_file_name.to_string());
    let default_ext_opt = default_path.extension();

    let mut output_file;
    if let Some(ext) = default_ext_opt {
        if let Some(file_name) = file_name_opt {
            output_file = OsString::from(file_name);
            output_file.push(".");
            output_file.push(language);
            output_file.push(".");
            output_file.push(ext)
        } else {
            output_file = OsString::from(&default_stem);
            output_file.push(".");
            output_file.push(language);
            output_file.push(".");
            output_file.push(ext)
        }
    } else {
        output_file = OsString::from(file_name_opt.as_deref().unwrap_or(&default_stem));
        output_file.push(".");
        output_file.push(language);
        output_file.push(".srt")
    };

    base_path.join(output_file)
}

#[derive(Debug)]
pub struct Downloaded {
    pub path: PathBuf,
    pub requests: i32,
    pub remaining: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_input_file() {
        assert_eq!(
            output_file(&PathBuf::from("/home/user"), &None, "default.ext", "en"),
            PathBuf::from("/home/user/default_en.ext")
        );
    }

    #[test]
    fn input_file_with_ext_from_default() {
        assert_eq!(
            output_file(
                &PathBuf::from("/home/user"),
                &Some(String::from("file")),
                "default.ext",
                "en"
            ),
            PathBuf::from("/home/user/file_en.ext")
        );
    }

    #[test]
    fn input_file_with_multiple_ext() {
        assert_eq!(
            output_file(
                &PathBuf::from("/home/user"),
                &Some(String::from("file.multiple")),
                "default.ext",
                "en"
            ),
            PathBuf::from("/home/user/file.multiple_en.ext")
        );
    }

    #[test]
    fn fallback_to_srt_if_default_has_no_extension() {
        assert_eq!(
            output_file(
                &PathBuf::from("/home/user"),
                &Some(String::from("file")),
                "default",
                "en"
            ),
            PathBuf::from("/home/user/file_en.srt")
        );
    }
}
