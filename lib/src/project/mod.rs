use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use log::{debug, info};

mod config;
mod download;
mod misc;
mod process;
mod rebuild;
mod state;

use crate::error::*;
use crate::threadupdater::*;
use crate::ui::*;

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

pub struct ChandlerProject {
    root_path: PathBuf,
    project_path: PathBuf,
    config_file_path: PathBuf,
    state_file_path: PathBuf,
    originals_path: PathBuf,
    config: ProjectConfig,
    state: ProjectState,
    thread: Option<Box<dyn ThreadUpdater>>,
}

#[derive(Debug)]
pub struct UpdateResult {
    pub was_updated: bool,
    pub is_dead: bool,
    pub new_post_count: u32,
    pub new_file_count: u32,
}

pub trait Project {
    fn update(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<UpdateResult, ChandlerError>;
    fn rebuild(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<(), ChandlerError>;
    fn save(&self) -> Result<(), ChandlerError>;
}

impl ChandlerProject {
    pub fn create(path: impl AsRef<Path>, url: &str) -> Result<Self, ChandlerError> {
        let root_path = path.as_ref().to_path_buf();
        let project_path = root_path.join(PROJECT_DIR_NAME);
        let originals_path = project_path.join(ORIGINALS_DIR_NAME);

        let config = ProjectConfig {
            parser: Parser::FourChan,
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

        fs::create_dir_all(&project_path).map_err(|err| {
            ChandlerError::CreateProject(Cow::Owned(format!("Cannot create project directory: {}", err)))
        })?;

        fs::create_dir_all(&originals_path).map_err(|err| {
            ChandlerError::CreateProject(Cow::Owned(format!("Cannot create originals directory: {}", err)))
        })?;

        let config_file_path = project_path.join(CONFIG_FILE_NAME);
        let state_file_path = project_path.join(STATE_FILE_NAME);

        // Save initial project config and state
        config.save(&config_file_path)?;
        state.save(&state_file_path)?;

        Ok(ChandlerProject {
            root_path: root_path,
            project_path: project_path,
            originals_path: originals_path,
            config_file_path,
            state_file_path,
            config,
            state,
            thread: None,
        })
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, ChandlerError> {
        let root_path = path.as_ref().to_path_buf();
        let project_path = root_path.join(PROJECT_DIR_NAME);
        let originals_path = project_path.join(ORIGINALS_DIR_NAME);

        let config_file_path = project_path.join(CONFIG_FILE_NAME);
        let state_file_path = project_path.join(STATE_FILE_NAME);

        // Load project config and state
        let config = ProjectConfig::load(&config_file_path)?;
        let state = ProjectState::load(&state_file_path)?;

        // Try to load current thread
        let thread = config
            .parser
            .create_thread_updater_from(&root_path.join(THREAD_FILE_NAME))
            .ok();

        Ok(ChandlerProject {
            root_path: root_path,
            project_path: project_path,
            originals_path: originals_path,
            config_file_path,
            state_file_path,
            config,
            state,
            thread,
        })
    }

    pub fn load_or_create(path: impl AsRef<Path>, url: &str) -> Result<Self, ChandlerError> {
        let root_path = path.as_ref().to_path_buf();
        let project_path = root_path.join(PROJECT_DIR_NAME);

        if project_path.exists() {
            Self::load(path)
        } else {
            Self::create(path, url)
        }
    }

    pub fn save_state(&self) -> Result<(), ChandlerError> {
        self.state.save(&self.state_file_path)?;

        Ok(())
    }

    pub fn write_thread(&self) -> Result<(), ChandlerError> {
        let thread_file_path = self.root_path.join(THREAD_FILE_NAME);
        info!("Writing thread HTML: {}", thread_file_path.display());

        if let Some(thread) = self.thread.as_ref() {
            thread.write_file(&thread_file_path)?;
        }

        Ok(())
    }
}

impl Project for ChandlerProject {
    fn update(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<UpdateResult, ChandlerError> {
        // Get unix timestamp
        let now = Utc::now();
        let unix_now = now.timestamp();

        // Construct filename
        let filename = format!("{}.html", unix_now);
        let thread_file_path = self.originals_path.join(&filename);

        let url = &self.config.url;

        info!("BEGIN UPDATE: {}", url);

        ui_handler.event(&UiEvent::UpdateStart {
            thread_url: url.clone(),
            destination: self.root_path.clone(),
        });

        // Download new thread HTML.
        let result = download_file(url, &thread_file_path, self.state.last_modified, ui_handler)?;

        let result = match result {
            DownloadResult::Success(last_modified) => {
                // Update last modified timestamp.
                self.state.last_modified = last_modified;

                // Process the new HTML.
                let process_result = process_thread(self, &thread_file_path)?;
                let update_result = process_result.update_result;

                self.state.is_dead = update_result.is_archived;

                // Download linked files.
                download_linked_files(self, ui_handler)?;

                Ok(UpdateResult {
                    was_updated: true,
                    is_dead: self.state.is_dead,
                    new_post_count: update_result.new_post_count,
                    new_file_count: process_result.new_file_count,
                })
            }
            DownloadResult::NotModified => Ok(UpdateResult {
                was_updated: false,
                is_dead: self.state.is_dead,
                new_post_count: 0,
                new_file_count: 0,
            }),
            DownloadResult::NotFound => {
                // If thread returned 404, mark it as dead.
                self.state.is_dead = true;

                Ok(UpdateResult {
                    was_updated: false,
                    is_dead: self.state.is_dead,
                    new_post_count: 0,
                    new_file_count: 0,
                })
            }
            DownloadResult::Error(_, description) => Err(ChandlerError::Download(Cow::Owned(description))),
        };

        info!("END UPDATE");

        if let Ok(result) = &result {
            ui_handler.event(&UiEvent::UpdateComplete {
                was_updated: result.was_updated,
                new_post_count: result.new_post_count,
                new_file_count: result.new_file_count,
            });
        }

        result
    }

    fn rebuild(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<(), ChandlerError> {
        rebuild_thread(self, ui_handler)
    }

    fn save(&self) -> Result<(), ChandlerError> {
        self.save_state()?;

        Ok(())
    }
}
