use std::path::{Path, PathBuf};

use log::*;

pub fn download_thread(url: &str, thread_path: &Path) -> Result<PathBuf, std::io::Error> {
    use std::fs::{self, File};

    use chrono::Utc;
    use reqwest;

    // Construct paths
    let project_path = thread_path.join(".chandler");
    let originals_path = project_path.join("originals");

    // Create directory if it's missing
    fs::create_dir_all(&originals_path)?;

    // Get unix timestamp
    let now = Utc::now();
    let unix_now = now.timestamp();

    // Construct filename
    let filename = originals_path.join(format!("{}.html", unix_now));

    debug!("Downloading thread from {} to file: {}", url, filename.to_str().unwrap_or("<CANNOT TO STRING>"));

    {
        // Create file for writing
        let mut thread_file = File::create(&filename)?;

        // Fetch catalog data from API and write it to the file
        reqwest::get(url).unwrap().copy_to(&mut thread_file).unwrap();
    }

    Ok(filename)
}
