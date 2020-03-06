use std::borrow::Cow;
use std::path::{Path, PathBuf};

use serde_derive::{Deserialize, Serialize};

mod misc;
mod rebuild;

use crate::error::*;

use self::misc::*;
use self::rebuild::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadConfig {
    parser: String,
    url: String,
    download_extensions: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkState {
    failed: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectState {
    last_modified: String,
    is_dead: bool,
    links: LinkState,
}

pub struct ChandlerProject {
    root_path: PathBuf,
    project_path: PathBuf,
    originals_path: PathBuf,
}

pub trait Project {
    fn update(&self) -> Result<PathBuf, ChandlerError>;
    fn rebuild(&self) -> Result<PathBuf, ChandlerError>;
}

impl ChandlerProject {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ChandlerError> {
        let root_path = path.as_ref().to_path_buf();
        let project_path = root_path.join(".chandler");
        let originals_path = project_path.join("originals");

        Ok(ChandlerProject {
            root_path: root_path,
            project_path: project_path,
            originals_path: originals_path,
        })
    }
}

impl Project for ChandlerProject {
    fn update(&self) -> Result<PathBuf, ChandlerError> {
        Err(ChandlerError::Other(Cow::Borrowed("Not implemented yet!")))
    }

    fn rebuild(&self) -> Result<PathBuf, ChandlerError> {
        let files = get_html_files(&self.originals_path).unwrap();
        let destination_file = self.root_path.join("thread2.html");

        rebuild_thread(files.as_slice(), &destination_file)?;

        Ok(destination_file)
    }
}
