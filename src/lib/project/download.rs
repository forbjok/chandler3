use std::path::Path;

use crate::util;

use super::*;

pub fn download_thread(url: &str, thread_file_path: &Path) -> Result<(), ChandlerError> {
    // Create file for writing
    let mut thread_file = util::create_file(&thread_file_path).map_err(|err| ChandlerError::CreateFile(err))?;

    // Fetch catalog data from API and write it to the file
    reqwest::blocking::get(url).unwrap().copy_to(&mut thread_file).unwrap();

    Ok(())
}
