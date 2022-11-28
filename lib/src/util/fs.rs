use std::fs;
use std::path::Path;

use anyhow::Context;

pub fn create_file(path: impl AsRef<Path>) -> Result<fs::File, anyhow::Error> {
    let path = path.as_ref();

    fs::File::create(path).with_context(|| format!("Error creating file: {}", path.display()))
}

pub fn open_file(path: impl AsRef<Path>) -> Result<fs::File, anyhow::Error> {
    let path = path.as_ref();

    fs::File::open(path).with_context(|| format!("Error opening file: {}", path.display()))
}

pub fn create_parent_dir(path: impl AsRef<Path>) -> Result<(), anyhow::Error> {
    if let Some(parent_dir_path) = path.as_ref().parent() {
        fs::create_dir_all(parent_dir_path)
            .with_context(|| format!("Error creating path: {}", parent_dir_path.display()))?;
    }

    Ok(())
}
