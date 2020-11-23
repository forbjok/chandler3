use std::collections::HashSet;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use log::{debug, info};

use crate::error::*;
use crate::threadupdater::{CreateThreadUpdater, ThreadUpdater};
use crate::ui::*;

use super::*;

#[derive(Debug)]
pub struct UpdateResult {
    pub was_updated: bool,
    pub last_modified: Option<DateTime<Utc>>,
    pub is_dead: bool,
    pub new_post_count: u32,
    pub new_file_count: u32,
}

pub fn update_thread(
    root_path: &Path,
    original_thread: &mut Option<Box<dyn ThreadUpdater>>,
    url: &str,
    last_modified: Option<DateTime<Utc>>,
    originals_path: &Path,
    download_extensions: &HashSet<String>,
    parser: &dyn CreateThreadUpdater,
    ui_handler: &mut dyn ChandlerUiHandler,
) -> Result<UpdateResult, ChandlerError> {
    // Get unix timestamp
    let now = Utc::now();
    let unix_now = now.timestamp();

    // Construct filename
    let filename = format!("{}.html", unix_now);
    let thread_file_path = originals_path.join(&filename);

    info!("BEGIN UPDATE: {}", url);

    ui_handler.event(&UiEvent::UpdateStart {
        thread_url: url.to_owned(),
        destination: root_path.to_path_buf(),
    });

    // Download new thread HTML.
    let result = download_file(url, &thread_file_path, last_modified, ui_handler)?;

    let result = match result {
        DownloadResult::Success { last_modified } => {
            // Process the new HTML.
            let process_result = process_thread(original_thread, &thread_file_path, url, download_extensions, parser)?;

            let update_result = process_result.update_result;

            Ok(UpdateResult {
                was_updated: true,
                last_modified,
                is_dead: update_result.is_archived,
                new_post_count: update_result.new_post_count,
                new_file_count: process_result.new_unprocessed_links.len() as u32,
            })
        }
        DownloadResult::NotModified => Ok(UpdateResult {
            was_updated: false,
            last_modified: None,
            is_dead: false,
            new_post_count: 0,
            new_file_count: 0,
        }),
        DownloadResult::NotFound => {
            // If thread returned 404, mark it as dead.
            Ok(UpdateResult {
                was_updated: false,
                last_modified: None,
                is_dead: true,
                new_post_count: 0,
                new_file_count: 0,
            })
        }
        DownloadResult::Error {
            status_code,
            description,
        } => Err(ChandlerError::DownloadHttpStatus {
            status_code,
            description: description.into(),
        }),
    };

    info!("END UPDATE");

    if let Ok(result) = &result {
        ui_handler.event(&UiEvent::UpdateComplete {
            was_updated: result.was_updated,
            new_post_count: result.new_post_count,
            new_file_count: result.new_file_count,
        });
    }

    result
}
