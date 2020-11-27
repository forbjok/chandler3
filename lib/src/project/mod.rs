use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use log::info;

pub mod common;
mod v2;
mod v3;

use common::LinkInfo;

use crate::error::*;
use crate::threadupdater::{ParserType, ThreadUpdater};
use crate::ui::*;

const DEFAULT_DOWNLOAD_EXTENSIONS: &[&str] = &["css", "gif", "ico", "jpg", "png", "webm"];

#[derive(Clone, Copy, Debug)]
pub enum ProjectFormat {
    V2,
    V3,
}

#[derive(Debug)]
pub struct ProjectUpdateResult {
    pub was_updated: bool,
    pub is_dead: bool,
    pub new_post_count: u32,
    pub new_file_count: u32,
}

pub struct ProjectState {
    pub root_path: PathBuf,
    pub thread_file_path: PathBuf,
    pub originals_path: PathBuf,
    pub thread_url: String,
    pub download_extensions: BTreeSet<String>,
    pub parser: ParserType,
    pub link_path_generator: Box<dyn LinkPathGenerator>,
    pub thread: Option<Box<dyn ThreadUpdater>>,
    pub last_modified: Option<DateTime<Utc>>,
    pub is_dead: bool,
    pub new_links: Vec<LinkInfo>,
    pub failed_links: Vec<LinkInfo>,
}

pub struct ProjectOptions {
    /// Project format to use when creating a new project.
    pub format: ProjectFormat,
}

pub trait LinkPathGenerator {
    fn generate_path(&self, url: &str) -> Result<Option<String>, ChandlerError>;
}

pub trait Project {
    fn update(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<ProjectUpdateResult, ChandlerError>;
    fn download_content(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<(), ChandlerError>;
    fn rebuild(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<(), ChandlerError>;
    fn save(&self) -> Result<(), ChandlerError>;
}

pub trait ProjectLoader {
    type P: Project;

    fn create(path: &Path, url: &str) -> Result<Self::P, ChandlerError>;
    fn load(path: &Path) -> Result<Self::P, ChandlerError>;
    fn exists_at(path: &Path) -> bool;
}

impl Default for ProjectOptions {
    fn default() -> Self {
        Self {
            format: ProjectFormat::V3,
        }
    }
}

impl ProjectState {
    pub fn write_thread(&self) -> Result<(), ChandlerError> {
        let thread_file_path = self.root_path.join(&self.thread_file_path);
        info!("Writing thread HTML: {}", thread_file_path.display());

        if let Some(thread) = self.thread.as_ref() {
            thread.write_file(&thread_file_path)?;
        }

        Ok(())
    }
}

pub fn exists_at(path: impl AsRef<Path>) -> Option<ProjectFormat> {
    let path = path.as_ref();

    if v3::V3Project::exists_at(path) {
        Some(ProjectFormat::V3)
    } else if v2::V2Project::exists_at(path) {
        Some(ProjectFormat::V2)
    } else {
        None
    }
}

pub fn load(path: impl AsRef<Path>) -> Result<Box<dyn Project>, ChandlerError> {
    let path = path.as_ref();

    if v3::V3Project::exists_at(path) {
        Ok(Box::new(v3::V3Project::load(path)?))
    } else if v2::V2Project::exists_at(path) {
        Ok(Box::new(v2::V2Project::load(path)?))
    } else {
        Err(ChandlerError::LoadProject("No project found".into()))
    }
}

pub fn load_or_create(
    path: impl AsRef<Path>,
    url: &str,
    project_options: ProjectOptions,
) -> Result<Box<dyn Project>, ChandlerError> {
    let path = path.as_ref();

    if exists_at(path).is_some() {
        load(path)
    } else {
        Ok(match project_options.format {
            ProjectFormat::V2 => Box::new(v2::V2Project::create(path, url)?),
            ProjectFormat::V3 => Box::new(v3::V3Project::create(path, url)?),
        })
    }
}
