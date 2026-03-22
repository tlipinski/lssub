use crate::osb::download::download;
use crate::osb::get_download_link::get_download_link;
use crate::osb::osb_client::OsbClient;
use crate::secret::retrieve_token;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{ChangeStatus, Multi, SubtitleDownloaded};
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
}

impl Downloader {
    pub fn new(base_path: PathBuf, file_name_opt: Option<String>) -> Self {
        Downloader {
            base_path,
            file_name_opt,
        }
    }

    pub async fn download(&self, file_id: i64, language: &str) -> Result<Action> {
        info!("Downloading subs file: {file_id:?}");

        let token_opt = retrieve_token().await?;

        let download_link_response = get_download_link(OsbClient::default(), token_opt, file_id)
            .await
            .map_err(|e| {
                error!("Downloading subs failed: {e}");
                e
            })?;

        debug!("Download link response: {:?}", download_link_response);
        debug!("Base path: {}", self.base_path.display());
        debug!("File name: {:?}", self.file_name_opt);

        let content = download(download_link_response.link).await.map_err(|e| {
            error!("Downloading subs failed: {e}");
            e
        })?;

        let output_file = output_file(
            &self.base_path,
            self.file_name_opt.as_deref(),
            download_link_response.file_name.as_str(),
            language,
        );

        debug!("Output file: {}", output_file.display());

        tokio::fs::write(output_file.clone(), content)
            .await
            .map_err(|e| {
                Error::msg(format!(
                    "Error saving subtitle file {}: {}",
                    output_file.display(),
                    e
                ))
            })?;

        Ok(Multi(vec![
            SubtitleDownloaded(Downloaded {
                path: output_file.clone(),
                requests: download_link_response.requests,
                remaining: download_link_response.remaining,
            }),
            ChangeStatus(format!(
                "Downloaded to {}",
                output_file.as_os_str().to_str().unwrap()
            )),
        ]))
    }
}

fn output_file(
    base_path: &Path,
    file_name_opt: Option<&str>,
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
        } else {
            output_file = OsString::from(&default_stem);
        }
        output_file.push(".");
        output_file.push(language);
        output_file.push(".");
        output_file.push(ext);
    } else {
        output_file = OsString::from(file_name_opt.as_deref().unwrap_or(&default_stem));
        output_file.push(".");
        output_file.push(language);
        output_file.push(".srt");
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
            output_file(&PathBuf::from("/home/user"), None, "default.ext", "en"),
            PathBuf::from("/home/user/default.en.ext")
        );
    }

    #[test]
    fn input_file_with_ext_from_default() {
        assert_eq!(
            output_file(
                &PathBuf::from("/home/user"),
                Some("file"),
                "default.ext",
                "en"
            ),
            PathBuf::from("/home/user/file.en.ext")
        );
    }

    #[test]
    fn input_file_with_multiple_ext() {
        assert_eq!(
            output_file(
                &PathBuf::from("/home/user"),
                Some("file.multiple"),
                "default.ext",
                "en"
            ),
            PathBuf::from("/home/user/file.multiple.en.ext")
        );
    }

    #[test]
    fn fallback_to_srt_if_default_has_no_extension() {
        assert_eq!(
            output_file(&PathBuf::from("/home/user"), Some("file"), "default", "en"),
            PathBuf::from("/home/user/file.en.srt")
        );
    }
}
