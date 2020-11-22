use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use log::{debug, info};

use crate::util;
use crate::progress::*;

use super::*;

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
    progress_callback_handler: &mut dyn ChandlerProgressCallbackHandler,
) -> Result<DownloadResult, ChandlerError> {
    info!("Download starting: '{}' (to '{}')", url, path.display());

    progress_callback_handler.progress(&ProgressEvent::DownloadStart(DownloadStartInfo {
        url : url.to_owned(),
        destination: path.to_path_buf(),
    }));

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

        progress_callback_handler.progress(&ProgressEvent::DownloadFileInfo(DownloadFileInfo {
            size: response.content_length(),
        }));

        progress_callback_handler.progress(&ProgressEvent::DownloadProgress(DownloadProgressInfo {
            bytes_downloaded: 0,
        }));

        // Create file for writing.
        let mut file = util::create_file(&path).map_err(|err| ChandlerError::CreateFile(err))?;

        let mut bytes_downloaded: usize = 0;

        // Copy response content to file.
        'copy: loop {
            use std::io::{Read, Write};

            let mut buf: [u8; 1024] = [0; 1024];

            match response.read(&mut buf) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        break 'copy;
                    }
                    bytes_downloaded += bytes_read;

                    progress_callback_handler.progress(&ProgressEvent::DownloadProgress(DownloadProgressInfo {
                        bytes_downloaded: bytes_downloaded as u64,
                    }));

                    file.write(&buf[..bytes_read]).map_err(|err| ChandlerError::Download(err.to_string().into()))?;
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
        Ok(_) => progress_callback_handler.progress(&ProgressEvent::DownloadComplete(DownloadCompleteInfo {
            result: DownloadCompleteResult::Success,
        })),
        Err(_) => progress_callback_handler.progress(&ProgressEvent::DownloadComplete(DownloadCompleteInfo {
            result: DownloadCompleteResult::Error,
        })),
    };

    result
}

/// Download all links for this project.
pub fn download_linked_files(project: &mut ChandlerProject, cancel: Arc<AtomicBool>, progress_callback_handler: &mut dyn ChandlerProgressCallbackHandler) -> Result<(), ChandlerError> {
    let mut unprocessed_links: Vec<LinkInfo> = Vec::new();
    unprocessed_links.append(&mut project.state.links.unprocessed);

    loop {
        // If cancellation has been requested, break out immediately.
        if cancel.load(Ordering::SeqCst) {
            break;
        }

        if unprocessed_links.is_empty() {
            break;
        }

        let link_info = unprocessed_links.remove(0);

        let path = project.root_path.join(&link_info.path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();

        if let Err(err) = download_file(&link_info.url, &path, None, progress_callback_handler) {
            debug!("Error downloading link: {}", err.to_string());

            project.state.links.failed.push(link_info.url.clone());
            project.state.links.unprocessed.push(link_info);
        }
    }

    project.state.links.unprocessed.append(&mut unprocessed_links);

    Ok(())
}
