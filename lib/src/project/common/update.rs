use chrono::Utc;
use log::info;

use crate::error::*;
use crate::project::ProjectState;
use crate::ui::*;

use super::*;

#[derive(Debug)]
pub struct UpdateResult {
    pub was_updated: bool,
    pub new_post_count: u32,
    pub new_link_count: u32,
}

pub fn update_thread(
    state: &mut ProjectState,
    ui_handler: &mut dyn ChandlerUiHandler,
) -> Result<UpdateResult, ChandlerError> {
    // Get unix timestamp
    let now = Utc::now();
    let unix_now = now.timestamp();

    // Construct filename
    let filename = format!("{}.html", unix_now);
    let new_thread_file_path = state.originals_path.join(&filename);

    let result = (|| {
        let url = &state.thread_url;

        info!("BEGIN UPDATE: {}", url);

        ui_handler.event(&UiEvent::UpdateStart {
            thread_url: url.to_owned(),
            destination: state.root_path.to_path_buf(),
        });

        // Download new thread HTML.
        let result = download_file(&url, &new_thread_file_path, state.last_modified, ui_handler)?;

        match result {
            DownloadResult::Success { last_modified } => {
                // Process the new HTML.
                let process_result = process_thread(state, &new_thread_file_path)?;

                let update_result = process_result.update_result;

                // If thread is archived, mark it as dead.
                state.is_dead = update_result.is_archived;

                // Update last modified date in project state.
                state.last_modified = last_modified;

                Ok(UpdateResult {
                    was_updated: true,
                    new_post_count: update_result.new_post_count,
                    new_link_count: process_result.new_file_count,
                })
            }
            DownloadResult::NotModified => Ok(UpdateResult {
                was_updated: false,
                new_post_count: 0,
                new_link_count: 0,
            }),
            DownloadResult::NotFound => {
                // If thread returned 404, mark it as dead.
                state.is_dead = true;

                Ok(UpdateResult {
                    was_updated: false,
                    new_post_count: 0,
                    new_link_count: 0,
                })
            }
            DownloadResult::OtherHttpError {
                status_code,
                description,
            } => Err(ChandlerError::Download(DownloadError::Http {
                code: status_code,
                description: description.into(),
            })),
        }
    })();

    info!("END UPDATE");

    match &result {
        Ok(result) => ui_handler.event(&UiEvent::UpdateComplete {
            was_updated: result.was_updated,
            new_post_count: result.new_post_count,
            new_file_count: result.new_link_count,
        }),
        Err(err) => ui_handler.event(&UiEvent::UpdateError {
            description: err.to_string(),
        }),
    };

    result
}
