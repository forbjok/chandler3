use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use log::{debug, info};

use crate::ui::*;
use crate::util;

use super::*;

const BUF_SIZE: usize = 65535;

#[derive(Debug)]
pub enum DownloadResult {
    Success(Option<DateTime<Utc>>),
    NotModified,
    NotFound,
    Error(u16, String),
}

pub fn download_file(
    url: &str,
    path: &Path,
    if_modified_since: Option<DateTime<Utc>>,
    ui_handler: &mut dyn ChandlerUiHandler,
) -> Result<DownloadResult, ChandlerError> {
    info!("Download starting: '{}' (to '{}')", url, path.display());

    ui_handler.event(&UiEvent::DownloadFileStart {
        url: url.to_owned(),
        destination: path.to_path_buf(),
    });

    let result = (|| {
        let client = reqwest::blocking::Client::builder()
            .user_agent("Chandler3")
            .build()
            .unwrap();

        // Download the thread HTML.
        let mut request = client.get(url);

        // If specified, add If-Modified-Since header.
        if let Some(if_modified_since) = if_modified_since {
            request = request.header(reqwest::header::IF_MODIFIED_SINCE, &if_modified_since.to_rfc2822());
        }

        // Send request and get response.
        let mut response = request
            .send()
            .map_err(|err| ChandlerError::Download(err.to_string().into()))?;

        let status = response.status();

        if !status.is_success() {
            let status_code: u16 = status.into();

            info!("Download failed: '{}' (status code: {})", url, status_code);

            return match status_code {
                304 => Ok(DownloadResult::NotModified),
                404 => Ok(DownloadResult::NotFound),
                _ => Ok(DownloadResult::Error(status_code, status.to_string())),
            };
        }

        ui_handler.event(&UiEvent::DownloadFileInfo {
            size: response.content_length(),
        });

        ui_handler.event(&UiEvent::DownloadFileProgress { bytes_downloaded: 0 });

        // Create file for writing.
        let mut file = util::create_file(&path).map_err(|err| ChandlerError::CreateFile(err))?;

        let mut bytes_downloaded: usize = 0;

        // Copy response content to file.
        'copy: loop {
            use std::io::{Read, Write};

            let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];

            match response.read(&mut buf) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        break 'copy;
                    }
                    bytes_downloaded += bytes_read;

                    ui_handler.event(&UiEvent::DownloadFileProgress {
                        bytes_downloaded: bytes_downloaded as u64,
                    });

                    file.write(&buf[..bytes_read])
                        .map_err(|err| ChandlerError::Download(err.to_string().into()))?;
                }
                Err(err) => return Err(ChandlerError::Download(err.to_string().into())),
            }
        }

        let last_modified: Option<DateTime<Utc>> =
            if let Some(value) = response.headers().get(reqwest::header::LAST_MODIFIED) {
                let value_str = value.to_str().unwrap();

                let last_modified = DateTime::parse_from_rfc2822(value_str)
                    .map_err(|err| ChandlerError::Download(Cow::Owned(err.to_string())))?;

                Some(last_modified.into())
            } else {
                None
            };

        info!("Download completed: '{}'", url);
        Ok(DownloadResult::Success(last_modified))
    })();

    // Report download complete progress event.
    match result {
        Ok(_) => ui_handler.event(&UiEvent::DownloadFileComplete(DownloadFileCompleteResult::Success)),
        Err(_) => ui_handler.event(&UiEvent::DownloadFileComplete(DownloadFileCompleteResult::Error)),
    };

    result
}

/// Download all links for this project.
pub fn download_linked_files(
    project: &mut ChandlerProject,
    ui_handler: &mut dyn ChandlerUiHandler,
) -> Result<(), ChandlerError> {
    let mut unprocessed_links: Vec<LinkInfo> = Vec::new();
    unprocessed_links.append(&mut project.state.links.unprocessed);

    // Report download start.
    ui_handler.event(&UiEvent::DownloadStart {
        file_count: unprocessed_links.len() as u32,
    });

    let mut files_processed: u32 = 0;
    let mut files_downloaded: u32 = 0;
    let mut files_failed: u32 = 0;

    loop {
        // If cancellation has been requested, break out immediately.
        if ui_handler.is_cancelled() {
            break;
        }

        if unprocessed_links.is_empty() {
            break;
        }

        let link_info = unprocessed_links.remove(0);

        let path = project.root_path.join(&link_info.path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();

        if let Err(err) = download_file(&link_info.url, &path, None, ui_handler) {
            debug!("Error downloading link: {}", err.to_string());

            project.state.links.failed.push(link_info.url.clone());
            project.state.links.unprocessed.push(link_info);

            files_failed += 1;
        } else {
            files_downloaded += 1;
        }

        files_processed += 1;

        // Report download progress.
        ui_handler.event(&UiEvent::DownloadProgress { files_processed });
    }

    project.state.links.unprocessed.append(&mut unprocessed_links);

    // Report download complete.
    ui_handler.event(&UiEvent::DownloadComplete {
        files_downloaded,
        files_failed,
    });

    Ok(())
}
