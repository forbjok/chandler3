use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use tracing::error;
use tracing::info;

use crate::error::*;
use crate::project::ProjectState;
use crate::ui::*;
use crate::util;

const BUF_SIZE: usize = 65535;

static USER_AGENT: Lazy<String> = Lazy::new(|| {
    let os = os_info::get();

    format!(
        "Mozilla/5.0 ({} {}; {}) Chandler/{}",
        os.os_type(),
        os.version(),
        os.bitness(),
        env!("CARGO_PKG_VERSION")
    )
});

#[derive(Debug)]
pub enum DownloadResult {
    Success { last_modified: Option<DateTime<Utc>> },
    NotModified,
    NotFound,
    OtherHttpError { status_code: u16, description: String },
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
            .user_agent(&*USER_AGENT)
            .gzip(true)
            .build()
            .map_err(|err| ChandlerError::Other(format!("Error building HTTP client: {err}").into()))?;

        // Download the thread HTML.
        let mut request = client.get(url);

        // If specified, add If-Modified-Since header.
        if let Some(if_modified_since) = if_modified_since {
            request = request.header(reqwest::header::IF_MODIFIED_SINCE, &if_modified_since.to_rfc2822());
        }

        // Send request and get response.
        let mut response = request
            .send()
            .map_err(|err| ChandlerError::Download(DownloadError::Network(err.to_string().into())))?;

        let status = response.status();

        if !status.is_success() {
            let status_code: u16 = status.into();

            info!("Download failed: '{}' (status code: {})", url, status_code);

            return match status_code {
                304 => Ok(DownloadResult::NotModified),
                404 => Ok(DownloadResult::NotFound),
                _ => Ok(DownloadResult::OtherHttpError {
                    status_code,
                    description: status.to_string(),
                }),
            };
        }

        ui_handler.event(&UiEvent::DownloadFileInfo {
            size: response.content_length(),
        });

        ui_handler.event(&UiEvent::DownloadFileProgress { bytes_downloaded: 0 });

        // Create file for writing.
        let mut file = util::create_file(path).map_err(ChandlerError::CreateFile)?;

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

                    file.write(&buf[..bytes_read]).map_err(ChandlerError::WriteFile)?;
                }
                Err(err) => return Err(ChandlerError::Download(DownloadError::Other(err.to_string().into()))),
            }
        }

        let last_modified: Option<DateTime<Utc>> =
            if let Some(value) = response.headers().get(reqwest::header::LAST_MODIFIED) {
                if let Ok(value_str) = value.to_str() {
                    let last_modified = DateTime::parse_from_rfc2822(value_str)
                        .map_err(|err| ChandlerError::Download(DownloadError::Other(err.to_string().into())))?;

                    Some(last_modified.into())
                } else {
                    None
                }
            } else {
                None
            };

        info!("Download completed: '{}'", url);
        Ok(DownloadResult::Success { last_modified })
    })();

    // Report download complete progress event.
    match result {
        Ok(_) => ui_handler.event(&UiEvent::DownloadFileComplete(DownloadFileCompleteResult::Success)),
        Err(_) => ui_handler.event(&UiEvent::DownloadFileComplete(DownloadFileCompleteResult::Error)),
    };

    result
}

/// Download all links for this project.
pub fn download_linked_content(
    state: &mut ProjectState,
    ui_handler: &mut dyn ChandlerUiHandler,
) -> Result<(), ChandlerError> {
    state.new_links.append(&mut state.failed_links);

    // Report download start.
    ui_handler.event(&UiEvent::DownloadStart {
        file_count: state.new_links.len() as u32,
    });

    let mut files_processed: u32 = 0;
    let mut files_downloaded: u32 = 0;
    let mut files_failed: u32 = 0;

    let download_path = &state.root_path;
    let new_links = &mut state.new_links;
    let failed_links = &mut state.failed_links;

    loop {
        // If cancellation has been requested, break out immediately.
        if ui_handler.is_cancelled() {
            break;
        }

        if new_links.is_empty() {
            break;
        }

        let link_info = new_links.remove(0);

        let url = &link_info.url;
        let path = download_path.join(&link_info.path);

        if let Some(parent_path) = path.parent() {
            fs::create_dir_all(parent_path).map_err(|err| {
                ChandlerError::Other(format!("Error creating path: {}: {err}", parent_path.display()).into())
            })?;
        }

        let mut if_modified_since: Option<DateTime<Utc>> = None;

        // If the file already exists, try to get its modification time
        // so that we can pass it to the request's If-Modified-Since header.
        if path.exists() {
            if let Ok(m) = fs::metadata(&path) {
                if let Ok(st) = m.modified() {
                    if_modified_since = Some(st.into());
                }
            }
        }

        let success = match download_file(url, &path, if_modified_since, ui_handler) {
            Ok(r) => match r {
                DownloadResult::Success { .. } => true,
                DownloadResult::NotModified => true,
                DownloadResult::NotFound => {
                    error!("File not found: {}", url);
                    false
                }
                DownloadResult::OtherHttpError {
                    status_code,
                    description,
                } => {
                    error!("Server returned HTTP error: {} {}", status_code, description);
                    false
                }
            },
            Err(err) => {
                error!("Error downloading link: {}", err.to_string());
                false
            }
        };

        files_processed += 1;

        if success {
            files_downloaded += 1;
        } else {
            failed_links.push(link_info);
            files_failed += 1;
        }

        // Report download progress.
        ui_handler.event(&UiEvent::DownloadProgress { files_processed });
    }

    // Report download complete.
    ui_handler.event(&UiEvent::DownloadComplete {
        files_downloaded,
        files_failed,
    });

    Ok(())
}
