use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use log::debug;

mod config;
mod download;
mod misc;
mod process;
mod rebuild;
mod state;

use crate::error::*;
use crate::threadparser::*;
use crate::util;

use self::config::*;
use self::download::*;
use self::misc::*;
use self::process::*;
use self::rebuild::*;
use self::state::*;

const PROJECT_DIR_NAME: &str = ".chandler";
const ORIGINALS_DIR_NAME: &str = "originals";
const CONFIG_FILE_NAME: &str = "thread.json";
const STATE_FILE_NAME: &str = "state.json";
const THREAD_FILE_NAME: &str = "thread.html";

#[derive(Debug)]
pub struct ChandlerProject<TP>
where
    TP: MergeableImageboardThread,
{
    root_path: PathBuf,
    project_path: PathBuf,
    originals_path: PathBuf,
    config: ProjectConfig,
    state: ProjectState,
    thread: Option<TP>,
}

pub trait Project {
    fn update(&mut self) -> Result<(), ChandlerError>;
    fn rebuild(&mut self) -> Result<(), ChandlerError>;
}

impl<TP> ChandlerProject<TP>
where
    TP: MergeableImageboardThread,
{
    pub fn create(path: impl AsRef<Path>, url: &str) -> Result<Self, ChandlerError> {
        let root_path = path.as_ref().to_path_buf();
        let project_path = root_path.join(PROJECT_DIR_NAME);
        let originals_path = project_path.join(ORIGINALS_DIR_NAME);

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
        config.save(project_path.join(CONFIG_FILE_NAME))?;
        state.save(project_path.join(STATE_FILE_NAME))?;

        Ok(ChandlerProject {
            root_path: root_path,
            project_path: project_path,
            originals_path: originals_path,
            config,
            state,
            thread: None,
        })
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, ChandlerError> {
        let root_path = path.as_ref().to_path_buf();
        let project_path = root_path.join(PROJECT_DIR_NAME);
        let originals_path = project_path.join(ORIGINALS_DIR_NAME);

        // Load project config and state
        let config = ProjectConfig::load(project_path.join(CONFIG_FILE_NAME))?;
        let state = ProjectState::load(project_path.join(STATE_FILE_NAME))?;

        // Try to load current thread
        let thread = TP::from_file(&root_path.join(THREAD_FILE_NAME)).ok();

        Ok(ChandlerProject {
            root_path: root_path,
            project_path: project_path,
            originals_path: originals_path,
            config,
            state,
            thread,
        })
    }

    pub fn write_thread(&self) -> Result<(), ChandlerError> {
        if let Some(thread) = self.thread.as_ref() {
            thread.write_file(&self.root_path.join(THREAD_FILE_NAME))?;
        }

        Ok(())
    }
}

impl<TP> Project for ChandlerProject<TP>
where
    TP: MergeableImageboardThread,
{
    fn update(&mut self) -> Result<(), ChandlerError> {
        // Get unix timestamp
        let now = Utc::now();
        let unix_now = now.timestamp();

        // Construct filename
        let filename = format!("{}.html", unix_now);
        let thread_file_path = self.originals_path.join(&filename);

        let url = &self.config.url;
        debug!("Downloading thread from {} to file: {}", url, &filename);

        // Download new HTML
        download_thread(url, &thread_file_path)?;

        // Process the new HTML
        process_thread(self, &thread_file_path)?;

        Ok(())
    }

    fn rebuild(&mut self) -> Result<(), ChandlerError> {
        rebuild_thread(self)
    }
}
