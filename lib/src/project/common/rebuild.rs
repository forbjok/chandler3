use std::collections::HashSet;
use std::path::PathBuf;

use crate::error::*;
use crate::project::*;
use crate::threadupdater::{CreateThreadUpdater, ThreadUpdater};
use crate::ui::*;

use super::*;

pub struct RebuildResult {
    pub thread: Box<dyn ThreadUpdater>,
}

pub fn rebuild_thread(
    path: &Path,
    thread_url: &str,
    extensions: &HashSet<String>,
    parser: &dyn CreateThreadUpdater,
    original_files: &[PathBuf],
    ui_handler: &mut dyn ChandlerUiHandler,
) -> Result<RebuildResult, ChandlerError> {
    // Report rebuild start.
    ui_handler.event(&UiEvent::RebuildStart {
        path: path.to_path_buf(),
        file_count: original_files.len() as u32,
    });

    let mut thread: Option<Box<dyn ThreadUpdater>> = None;
    let mut files_processed: u32 = 0;

    for file in original_files.iter() {
        let update_result = process_thread(&mut thread, file, thread_url, extensions, parser)?;

        // Report progress.
        files_processed += 1;
        ui_handler.event(&UiEvent::RebuildProgress { files_processed });
    }

    // Report rebuild complete.
    ui_handler.event(&UiEvent::RebuildComplete);

    Ok(RebuildResult {
        thread: thread.unwrap(),
    })
}
