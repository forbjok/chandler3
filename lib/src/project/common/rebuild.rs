use crate::error::*;
use crate::project::*;
use crate::ui::*;

use super::*;

pub fn rebuild_thread(state: &mut ProjectState, ui_handler: &mut dyn ChandlerUiHandler) -> Result<(), ChandlerError> {
    let original_files = get_html_files(&state.originals_path)
        .map_err(|err| ChandlerError::Other(format!("Error getting HTML files: {}", err).into()))?;

    // Report rebuild start.
    ui_handler.event(&UiEvent::RebuildStart {
        path: state.root_path.to_path_buf(),
        file_count: original_files.len() as u32,
    });

    // Clear existing thread object.
    state.thread = None;

    for (i, file) in original_files.iter().enumerate() {
        let _update_result = process_thread(state, file)?;

        // Report progress.
        ui_handler.event(&UiEvent::RebuildProgress {
            files_processed: i as u32,
        });
    }

    // Report rebuild complete.
    ui_handler.event(&UiEvent::RebuildComplete);

    Ok(())
}
