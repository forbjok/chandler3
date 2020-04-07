use std::borrow::Cow;

use crate::error::*;
use crate::threadparser::*;

use super::*;

pub fn rebuild_thread(project: &mut ChandlerProject) -> Result<(), ChandlerError> {
    let files = get_html_files(&project.originals_path)
        .map_err(|err| ChandlerError::Other(Cow::Owned(format!("Error getting HTML files: {}", err))))?;

    // Get file iterator
    let files_iter = files.iter();

    // Set thread to None to ensure thread is regenerated from scratch.
    project.thread = None;

    for file in files_iter {
        println!("FILE: {:?}", file);

        process_thread(project, file)?;
    }

    project.write_thread()?;

    Ok(())
}
