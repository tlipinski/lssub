use log::info;
use osb::download::download;
use osb::get_download_link::get_download_link;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;

pub async fn downloader_task(
    rx: Receiver<SubsDownload>,
    base_path: PathBuf,
    file_name_opt: Option<String>,
) -> anyhow::Result<()> {
    loop {
        match rx.recv() {
            Ok(subs_download) => {
                info!("Downloading: {subs_download:?}");

                let download_link_response = get_download_link(subs_download.file_id).await?;

                let content = download(download_link_response.link).await?;

                let output_file = output_file(
                    &base_path,
                    &file_name_opt,
                    download_link_response.file_name.as_str(),
                );

                fs::write(output_file, content)?;
            }
            Err(err) => {
                info!("Error: {err}");
                break Ok(());
            }
        }
    }
}

fn output_file(
    base_path: &PathBuf,
    file_name_opt: &Option<String>,
    default_file_name: &str,
) -> PathBuf {
    let ext = PathBuf::from(default_file_name).extension();

    let file_base = file_name_opt.as_deref().unwrap_or(&default_file_name);

    let file = Path::new(file_base).with_extension("srt");

    base_path.join(file)
}

#[derive(Debug)]
pub struct SubsDownload {
    pub file_id: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_input_file() {
        assert_eq!(
            output_file(&PathBuf::from("/home/user"), &None, "default.ext"),
            PathBuf::from("/home/user/default.ext")
        );
    }

    #[test]
    fn input_file_with_ext_from_default() {
        assert_eq!(
            output_file(
                &PathBuf::from("/home/user"),
                &Some(String::from("file")),
                "default.ext"
            ),
            PathBuf::from("/home/user/file.ext")
        );
    }
}
