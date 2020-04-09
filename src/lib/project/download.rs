use std::fs;
use std::path::Path;

use log::{error, debug};

use crate::util;

use super::*;

pub fn download_file(url: &str, path: &Path) -> Result<(), ChandlerError> {
    // Create file for writing.
    let mut file = util::create_file(&path).map_err(|err| ChandlerError::CreateFile(err))?;

    // Download the thread HTML.
    let mut response = reqwest::blocking::get(url).map_err(|err| ChandlerError::Download(err.to_string().into()))?;

    // Write it to the file.
    response
        .copy_to(&mut file)
        .map_err(|err| ChandlerError::Other(err.to_string().into()))?;

    Ok(())
}

/// Download all links for this project.
pub fn download_linked_files(project: &mut ChandlerProject) -> Result<(), ChandlerError> {
    let mut unprocessed_links: Vec<LinkInfo> = Vec::new();
    unprocessed_links.append(&mut project.state.links.unprocessed);

    for link_info in unprocessed_links.into_iter() {
        error!("Downloading {} to {} ...", link_info.url, link_info.path);

        let path = project.root_path.join(&link_info.path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();

        if let Err(err) = download_file(&link_info.url, &path) {
            error!("Error downloading link: {}", err.to_string());

            project.state.links.failed.push(link_info.url.clone());
            project.state.links.unprocessed.push(link_info);
        }
    }

    Ok(())
}
