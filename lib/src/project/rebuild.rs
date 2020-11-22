use std::borrow::Cow;

use crate::error::*;
use crate::progress::*;
use crate::threadparser::*;

use super::*;

pub fn rebuild_thread(
    project: &mut ChandlerProject,
    progress_callback_handler: &mut dyn ChandlerProgressCallbackHandler,
) -> Result<(), ChandlerError> {
    let files = get_html_files(&project.originals_path)
        .map_err(|err| ChandlerError::Other(Cow::Owned(format!("Error getting HTML files: {}", err))))?;

    // Report rebuild start.
    progress_callback_handler.progress(&ProgressEvent::RebuildStart {
        path: project.root_path.clone(),
        file_count: files.len() as u32,
    });

    // Get file iterator.
    let files_iter = files.iter();

    // Set thread to None to ensure thread is regenerated from scratch.
    project.thread = None;

    let mut files_processed: u32 = 0;

    for file in files_iter {
        process_thread(project, file)?;

        // Report progress.
        files_processed += 1;
        progress_callback_handler.progress(&ProgressEvent::RebuildProgress { files_processed });
    }

    project.write_thread()?;

    // Report rebuild complete.
    progress_callback_handler.progress(&ProgressEvent::RebuildComplete);

    Ok(())
}
