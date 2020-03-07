use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use log::debug;

mod config;
mod download;
mod misc;
mod rebuild;
mod state;

use crate::error::*;

use self::config::*;
use self::download::*;
use self::misc::*;
use self::rebuild::*;
use self::state::*;

#[derive(Debug)]
pub struct ChandlerProject {
    root_path: PathBuf,
    project_path: PathBuf,
    originals_path: PathBuf,
    config: ProjectConfig,
    state: ProjectState,
}

pub trait Project {
    fn update(&self) -> Result<PathBuf, ChandlerError>;
    fn rebuild(&self) -> Result<PathBuf, ChandlerError>;
}

impl ChandlerProject {
    pub fn create(path: impl AsRef<Path>, url: &str) -> Result<Self, ChandlerError> {
        let root_path = path.as_ref().to_path_buf();
        let project_path = root_path.join(".chandler");
        let originals_path = project_path.join("originals");

        let config = ProjectConfig {
            parser: "4chan".to_owned(),
            url: url.to_owned(),
            download_extensions: vec![
                "ico".to_owned(),
                "css".to_owned(),
                "png".to_owned(),
                "jpg".to_owned(),
                "gif".to_owned(),
                "webm".to_owned(),
            ],
        };

        let state: ProjectState = Default::default();

        fs::create_dir_all(&root_path).map_err(|err| {
            ChandlerError::CreateProject(Cow::Owned(format!("Cannot create project directory: {}", err)))
        })?;

        // Save initial project config and state
        config.save(project_path.join("thread.json"))?;
        state.save(project_path.join("state.json"))?;

        Ok(ChandlerProject {
            root_path: root_path,
            project_path: project_path,
            originals_path: originals_path,
            config,
            state,
        })
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, ChandlerError> {
        let root_path = path.as_ref().to_path_buf();
        let project_path = root_path.join(".chandler");
        let originals_path = project_path.join("originals");

        // Load project config and state
        let config = ProjectConfig::load(project_path.join("thread.json"))?;
        let state = ProjectState::load(project_path.join("state.json"))?;

        Ok(ChandlerProject {
            root_path: root_path,
            project_path: project_path,
            originals_path: originals_path,
            config,
            state,
        })
    }
}

impl Project for ChandlerProject {
    fn update(&self) -> Result<PathBuf, ChandlerError> {
        // Get unix timestamp
        let now = Utc::now();
        let unix_now = now.timestamp();

        // Construct filename
        let filename = format!("{}.html", unix_now);
        let thread_file_path = self.originals_path.join(&filename);

        let url = &self.config.url;
        debug!("Downloading thread from {} to file: {}", url, &filename);

        download_thread(url, &thread_file_path)?;

        Ok(thread_file_path)
    }

    fn rebuild(&self) -> Result<PathBuf, ChandlerError> {
        let files = get_html_files(&self.originals_path).unwrap();
        let destination_file = self.root_path.join("thread.html");

        rebuild_thread(files.as_slice(), &destination_file)?;

        Ok(destination_file)
    }
}
