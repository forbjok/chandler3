use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;

lazy_static! {
    static ref HTML_EXTENSION: &'static OsStr = OsStr::new("html");
}

pub fn get_html_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();

    for entry in dir.read_dir()? {
        let path = entry?.path();

        // Exclude everything that is not a file
        if !path.is_file() {
            continue;
        }

        // Exclude non-.html files
        if path.extension() != Some(*HTML_EXTENSION) {
            continue;
        }

        files.push(path);
    }

    // Sort files alphabetically
    files.sort();

    Ok(files)
}
