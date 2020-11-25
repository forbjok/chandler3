use std::path::PathBuf;

use crate::error::*;
use crate::project::*;
use crate::threadupdater::ThreadUpdater;
use crate::ui::*;

use super::*;

pub struct RebuildResult {
    pub thread: Box<dyn ThreadUpdater>,
}

pub fn rebuild_thread(
    config: &ProjectConfig,
    original_files: &[PathBuf],
    ui_handler: &mut dyn ChandlerUiHandler,
) -> Result<RebuildResult, ChandlerError> {
    // Report rebuild start.
    ui_handler.event(&UiEvent::RebuildStart {
        path: config.download_root_path.to_path_buf(),
        file_count: original_files.len() as u32,
    });

    let mut thread: Option<Box<dyn ThreadUpdater>> = None;

    for (i, file) in original_files.iter().enumerate() {
        let _update_result = process_thread(config, &mut thread, file)?;

        // Report progress.
        ui_handler.event(&UiEvent::RebuildProgress {
            files_processed: i as u32,
        });
    }

    // Report rebuild complete.
    ui_handler.event(&UiEvent::RebuildComplete);

    Ok(RebuildResult {
        thread: thread.ok_or_else(|| ChandlerError::Other("No thread produced during rebuild!".into()))?,
    })
}
