use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use tracing::debug;
use url::Url;

mod format;

use crate::error::*;
use crate::threadupdater::CreateThreadUpdater;
use crate::ui::*;
use crate::util::pid::PidLock;

use self::format as pf;

use super::common::*;
use super::*;

const PROJECT_DIR_NAME: &str = ".chandler";
const ORIGINALS_DIR_NAME: &str = "originals";
const CONFIG_FILE_NAME: &str = "thread.json";
const STATE_FILE_NAME: &str = "state.json";
const THREAD_FILE_NAME: &str = "thread.html";
const PID_FILE_NAME: &str = "chandler.pid";

pub struct V2Project {
    state: ProjectState,
    state_file_path: PathBuf,
    _pidlock: PidLock,
}

struct V2LinkPathGenerator {
    thread_url: Url,
}

impl V2LinkPathGenerator {
    pub fn new(thread_url: &str) -> Result<Self, ChandlerError> {
        let thread_url = Url::parse(thread_url)
            .map_err(|err| ChandlerError::Other(format!("Error parsing thread URL: {}", err).into()))?;

        Ok(Self { thread_url })
    }
}

impl ProjectLoader for V2Project {
    type P = V2Project;

    fn create(path: &Path, url: &str, parser: ParserType) -> Result<Self::P, ChandlerError> {
        let root_path = path.to_path_buf();
        let project_path = root_path.join(PROJECT_DIR_NAME);
        let originals_path = project_path.join(ORIGINALS_DIR_NAME);

        fs::create_dir_all(&project_path).map_err(|err| {
            ChandlerError::CreateProject(Cow::Owned(format!("Cannot create project directory: {}", err)))
        })?;

        fs::create_dir_all(&originals_path).map_err(|err| {
            ChandlerError::CreateProject(Cow::Owned(format!("Cannot create originals directory: {}", err)))
        })?;

        let pidlock = if let Some(pidlock) = acquire_pidlock(&root_path, PID_FILE_NAME) {
            pidlock
        } else {
            return Err(ChandlerError::CreateProject(
                "Could not acquire PID lock for project.".into(),
            ));
        };

        let config_file_path = project_path.join(CONFIG_FILE_NAME);
        let state_file_path = project_path.join(STATE_FILE_NAME);
        let thread_file_path = root_path.join(THREAD_FILE_NAME);

        let link_path_generator = V2LinkPathGenerator::new(url)?;

        let state = ProjectState {
            root_path,
            thread_file_path,
            originals_path,
            thread_url: url.to_owned(),
            download_extensions: DEFAULT_DOWNLOAD_EXTENSIONS.iter().map(|s| (*s).to_owned()).collect(),
            parser,
            link_path_generator: Box::new(link_path_generator),
            thread: None,
            is_dead: false,
            last_modified: None,
            new_links: Vec::new(),
            failed_links: Vec::new(),
            seen_links: HashSet::new(),
        };

        // Save initial project config and state.
        pf::Config::from(&state).save(config_file_path)?;
        pf::State::default().save(&state_file_path)?;

        let project = Self {
            state,
            state_file_path,
            _pidlock: pidlock,
        };

        project.save()?;

        Ok(project)
    }

    fn load(path: &Path) -> Result<Self::P, ChandlerError> {
        let root_path = path.to_path_buf();

        let pidlock = if let Some(pidlock) = acquire_pidlock(&root_path, PID_FILE_NAME) {
            pidlock
        } else {
            return Err(ChandlerError::CreateProject(
                "Could not acquire PID lock for project.".into(),
            ));
        };

        let project_path = root_path.join(PROJECT_DIR_NAME);
        let originals_path = project_path.join(ORIGINALS_DIR_NAME);

        let config_file_path = project_path.join(CONFIG_FILE_NAME);
        let state_file_path = project_path.join(STATE_FILE_NAME);
        let thread_file_path = root_path.join(THREAD_FILE_NAME);

        // Load project config and state.
        let config = pf::Config::load(config_file_path)?;
        let state = pf::State::load(&state_file_path)?;

        let parser: ParserType = config.parser.into();
        let link_path_generator = V2LinkPathGenerator::new(&config.url)?;

        // Try to load current thread.
        let thread = parser
            .create_thread_updater_from(&root_path.join(THREAD_FILE_NAME))
            .ok();

        // Pull failed (in V2 projects, this includes unprocessed) links out of project state.
        let failed_links: Vec<LinkInfo> = state
            .links
            .failed
            .into_iter()
            .filter_map(|url| {
                if let Ok(Some(path)) = link_path_generator.generate_path(&url) {
                    Some((url, path))
                } else {
                    None
                }
            })
            .map(|(url, path)| LinkInfo { url, path })
            .collect();

        let seen_links: HashSet<String> = failed_links.iter().map(|l| l.url.clone()).collect();

        let state = ProjectState {
            root_path,
            thread_file_path,
            originals_path,
            thread_url: config.url,
            download_extensions: config.download_extensions,
            parser,
            link_path_generator: Box::new(link_path_generator),
            thread,
            is_dead: state.is_dead,
            last_modified: state.last_modified,
            new_links: Vec::new(),
            failed_links,
            seen_links,
        };

        Ok(Self {
            state,
            state_file_path,
            _pidlock: pidlock,
        })
    }

    fn exists_at(path: &Path) -> bool {
        path.join(PROJECT_DIR_NAME).is_dir()
    }
}

impl V2Project {
    fn save_state(&self) -> Result<(), ChandlerError> {
        pf::State::from(&self.state).save(&self.state_file_path)?;

        Ok(())
    }
}

impl Project for V2Project {
    fn update(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<ProjectUpdateResult, ChandlerError> {
        let update_result = update_thread(&mut self.state, ui_handler)?;

        // Write thread HTML.
        self.state.write_thread()?;

        // Download links.
        self.download_content(ui_handler)?;

        Ok(ProjectUpdateResult {
            was_updated: update_result.was_updated,
            is_dead: self.state.is_dead,
            new_post_count: update_result.new_post_count,
            new_file_count: update_result.new_link_count,
        })
    }

    fn download_content(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<(), ChandlerError> {
        // Download linked content.
        download_linked_content(&mut self.state, ui_handler)?;

        self.save_state()?;

        Ok(())
    }

    fn rebuild(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<(), ChandlerError> {
        rebuild_thread(&mut self.state, ui_handler)?;

        // Write rebuilt thread to file.
        self.state.write_thread()?;

        Ok(())
    }

    fn save(&self) -> Result<(), ChandlerError> {
        self.save_state()?;

        Ok(())
    }

    fn get_path(&self) -> &Path {
        &self.state.root_path
    }
}

impl LinkPathGenerator for V2LinkPathGenerator {
    fn generate_path(&self, url: &str) -> Result<Option<String>, ChandlerError> {
        let url = self
            .thread_url
            .join(url)
            .map_err(|err| ChandlerError::Other(err.to_string().into()))?;

        if let Some(host) = url.host_str() {
            Ok(Some(format!("{}{}", host, url.path())))
        } else {
            debug!("No host found in url: {}", url);
            Ok(None)
        }
    }
}
