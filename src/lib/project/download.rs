use std::path::Path;

use crate::util;

use super::*;

pub fn download_thread(url: &str, thread_file_path: &Path) -> Result<(), ChandlerError> {
    // Create file for writing.
    let mut thread_file = util::create_file(&thread_file_path).map_err(|err| ChandlerError::CreateFile(err))?;

    // Download the thread HTML.
    let mut response = reqwest::blocking::get(url).map_err(|err| ChandlerError::Download(err.to_string().into()))?;

    // Write it to the file.
    response
        .copy_to(&mut thread_file)
        .map_err(|err| ChandlerError::Other(err.to_string().into()))?;

    Ok(())
}
