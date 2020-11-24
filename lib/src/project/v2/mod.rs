use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use log::info;

mod config;
mod state;

use crate::error::*;
use crate::html;
use crate::threadupdater::{CreateThreadUpdater, ThreadUpdater};
use crate::ui::*;

use self::config as cfg;
use self::state as st;

use super::common::*;
use super::*;

const PROJECT_DIR_NAME: &str = ".chandler";
const ORIGINALS_DIR_NAME: &str = "originals";
const CONFIG_FILE_NAME: &str = "thread.json";
const STATE_FILE_NAME: &str = "state.json";
const THREAD_FILE_NAME: &str = "thread.html";

pub struct V2Project {
    root_path: PathBuf,
    project_path: PathBuf,
    config_file_path: PathBuf,
    state_file_path: PathBuf,
    originals_path: PathBuf,
    config: cfg::ProjectConfig,
    state: st::ProjectState,
    thread: Option<Box<dyn ThreadUpdater>>,
}

impl ProjectLoader for V2Project {
    type P = V2Project;

    fn create(path: &Path, url: &str) -> Result<Self::P, ChandlerError> {
        let root_path = path.to_path_buf();
        let project_path = root_path.join(PROJECT_DIR_NAME);
        let originals_path = project_path.join(ORIGINALS_DIR_NAME);

        let config = cfg::ProjectConfig {
            parser: cfg::Parser::FourChan,
            url: url.to_owned(),
            download_extensions: vec![
                "ico".to_owned(),
                "css".to_owned(),
                "png".to_owned(),
                "jpg".to_owned(),
                "gif".to_owned(),
                "webm".to_owned(),
            ]
            .into_iter()
            .collect(),
        };

        let state: st::ProjectState = Default::default();

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

        Ok(Self {
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

    fn load(path: &Path) -> Result<Self::P, ChandlerError> {
        let root_path = path.to_path_buf();
        let project_path = root_path.join(PROJECT_DIR_NAME);
        let originals_path = project_path.join(ORIGINALS_DIR_NAME);

        let config_file_path = project_path.join(CONFIG_FILE_NAME);
        let state_file_path = project_path.join(STATE_FILE_NAME);

        // Load project config and state
        let config = cfg::ProjectConfig::load(&config_file_path)?;
        let state = st::ProjectState::load(&state_file_path)?;

        let parser: ParserType = config.parser.into();

        // Try to load current thread
        let thread = parser
            .create_thread_updater_from(&root_path.join(THREAD_FILE_NAME))
            .ok();

        Ok(Self {
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

    fn exists_at(path: &Path) -> bool {
        path.join(PROJECT_DIR_NAME).is_dir()
    }
}

impl V2Project {
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

impl Project for V2Project {
    fn update(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<ProjectUpdateResult, ChandlerError> {
        let mut update_result = update_thread(
            &self.get_config(),
            &mut self.thread,
            self.state.last_modified,
            ui_handler,
        )?;

        let new_file_count = update_result.new_links.len() as u32;

        if update_result.was_updated {
            // Update last modified date in project state.
            self.state.last_modified = update_result.last_modified;

            let thread_url = url::Url::parse(&self.config.url)
                .map_err(|err| ChandlerError::Other(format!("Error parsing thread URL: {}", err).into()))?;

            // Pull failed (in V2 projects, this includes unprocessed) links out of project state.
            let mut new_links: Vec<LinkInfo> = self
                .state
                .links
                .failed
                .drain(..)
                .filter_map(|url| {
                    if let Ok(Some(path)) = local_path_from_url(&url, &thread_url) {
                        Some((url, path))
                    } else {
                        None
                    }
                })
                .map(|(url, path)| LinkInfo { url, path })
                .collect();

            new_links.append(&mut update_result.new_links);

            let mut failed_links: Vec<LinkInfo> = Vec::new();

            // Download linked files.
            download_linked_files(&self.root_path, &mut new_links, &mut failed_links, ui_handler)?;

            // Add remaining new links to failed link, because this is how it
            // was handled in V2 projects.
            failed_links.append(&mut new_links);

            // Re-add remaining failed links.
            self.state
                .links
                .failed
                .extend(failed_links.into_iter().map(|li| li.url));
        }

        self.write_thread()?;

        Ok(ProjectUpdateResult {
            was_updated: update_result.was_updated,
            is_dead: update_result.is_dead,
            new_post_count: update_result.new_post_count,
            new_file_count,
        })
    }

    fn rebuild(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<(), ChandlerError> {
        let files = get_html_files(&self.originals_path)
            .map_err(|err| ChandlerError::Other(Cow::Owned(format!("Error getting HTML files: {}", err))))?;

        let result = rebuild_thread(&self.get_config(), &files, ui_handler)?;

        // Write rebuilt thread to file.
        self.write_thread()?;

        Ok(())
    }

    fn save(&self) -> Result<(), ChandlerError> {
        self.save_state()?;

        Ok(())
    }

    fn get_config(&self) -> ProjectConfig {
        ProjectConfig {
            download_root_path: self.root_path.clone(),
            originals_path: self.originals_path.clone(),
            thread_url: self.config.url.clone(),
            download_extensions: self.config.download_extensions.clone(),
            parser: self.config.parser.into(),
        }
    }
}
