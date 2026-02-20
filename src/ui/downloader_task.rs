use log::{debug, info};
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

                debug!("{:?}", download_link_response);
                debug!("{:?}", base_path);
                debug!("{:?}", file_name_opt);

                let content = download(download_link_response.link).await?;


                let output_file = output_file(
                    &base_path,
                    &file_name_opt,
                    download_link_response.file_name.as_str(),
                );

                debug!("out: {:?}", output_file);

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
    base_path: &Path,
    file_name_opt: &Option<String>,
    default_file_name: &str,
) -> PathBuf {
    let default_ext_opt = Path::new(default_file_name).extension();

    let mut output_file;
    if let Some(ext) = default_ext_opt {
        if let Some(file_name) = file_name_opt {
            output_file = OsString::from(file_name);
            output_file.push(".");
            output_file.push(ext)
        } else {
            output_file = OsString::from(default_file_name)
        }
    } else {
        output_file = OsString::from(file_name_opt.as_deref().unwrap_or(default_file_name));
        output_file.push(".srt")
    };

    base_path.join(output_file)
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

    #[test]
    fn input_file_with_multiple_ext() {
        assert_eq!(
            output_file(
                &PathBuf::from("/home/user"),
                &Some(String::from("file.multiple")),
                "default.ext"
            ),
            PathBuf::from("/home/user/file.multiple.ext")
        );
    }

    #[test]
    fn fallback_to_srt_if_default_has_no_extension() {
        assert_eq!(
            output_file(
                &PathBuf::from("/home/user"),
                &Some(String::from("file")),
                "default"
            ),
            PathBuf::from("/home/user/file.srt")
        );
    }
}
