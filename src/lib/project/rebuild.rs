use std::borrow::Cow;

use crate::error::*;
use crate::threadparser::*;

use super::*;

pub fn rebuild_thread<TP>(project: &mut ChandlerProject<TP>) -> Result<(), ChandlerError>
where
    TP: MergeableImageboardThread,
{
    let files = get_html_files(&project.originals_path)
        .map_err(|err| ChandlerError::Other(Cow::Owned(format!("Error getting HTML files: {}", err))))?;

    // Get file iterator
    let mut files_iter = files.iter();

    // Get the first file
    let first_file = files_iter
        .next()
        .ok_or_else(|| ChandlerError::Other(Cow::Owned("First file not found!".to_owned())))?;

    let first_thread = TP::from_file(first_file)?;

    project.thread = Some(first_thread);

    for file in files_iter {
        println!("FILE: {:?}", file);

        process_thread(project, file)?;
    }

    project.write_thread()?;

    Ok(())
}
