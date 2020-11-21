use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use log::{debug, info};

use crate::util;

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
) -> Result<DownloadResult, ChandlerError> {
    info!("Download starting: '{}' (to '{}')", url, path.display());

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

    // Create file for writing.
    let mut file = util::create_file(&path).map_err(|err| ChandlerError::CreateFile(err))?;

    // Write it to the file.
    response
        .copy_to(&mut file)
        .map_err(|err| ChandlerError::Other(err.to_string().into()))?;

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
}

/// Download all links for this project.
pub fn download_linked_files(project: &mut ChandlerProject, cancel: Arc<AtomicBool>) -> Result<(), ChandlerError> {
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

        if let Err(err) = download_file(&link_info.url, &path, None) {
            debug!("Error downloading link: {}", err.to_string());

            project.state.links.failed.push(link_info.url.clone());
            project.state.links.unprocessed.push(link_info);
        }
    }

    project.state.links.unprocessed.append(&mut unprocessed_links);

    Ok(())
}
