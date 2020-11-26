use std::collections::HashSet;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use log::info;

pub mod common;
mod v2;
mod v3;

use common::LinkInfo;

use crate::error::*;
use crate::threadupdater::{ParserType, ThreadUpdater};
use crate::ui::*;

lazy_static! {
    static ref DEFAULT_DOWNLOAD_EXTENSIONS: HashSet<String> = vec![
        "ico".to_owned(),
        "css".to_owned(),
        "png".to_owned(),
        "jpg".to_owned(),
        "gif".to_owned(),
        "webm".to_owned(),
    ]
    .into_iter()
    .collect();
}

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
    pub download_extensions: HashSet<String>,
    pub parser: ParserType,
    pub thread: Option<Box<dyn ThreadUpdater>>,
    pub last_modified: Option<DateTime<Utc>>,
    pub is_dead: bool,
    pub new_links: Vec<LinkInfo>,
    pub failed_links: Vec<LinkInfo>,
}

pub trait Project {
    fn update(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<ProjectUpdateResult, ChandlerError>;
    fn download_links(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<(), ChandlerError>;
    fn rebuild(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<(), ChandlerError>;
    fn save(&self) -> Result<(), ChandlerError>;
}

pub trait ProjectLoader {
    type P: Project;

    fn create(path: &Path, url: &str) -> Result<Self::P, ChandlerError>;
    fn load(path: &Path) -> Result<Self::P, ChandlerError>;
    fn exists_at(path: &Path) -> bool;
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

pub fn exists_at(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();

    v3::V3Project::exists_at(path) || v2::V2Project::exists_at(path)
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

pub fn load_or_create_format(
    path: impl AsRef<Path>,
    url: &str,
    create_project_format: ProjectFormat,
) -> Result<Box<dyn Project>, ChandlerError> {
    let path = path.as_ref();

    if exists_at(path) {
        load(path)
    } else {
        Ok(match create_project_format {
            ProjectFormat::V2 => Box::new(v2::V2Project::create(path, url)?),
            ProjectFormat::V3 => Box::new(v3::V3Project::create(path, url)?),
        })
    }
}

pub fn load_or_create(path: impl AsRef<Path>, url: &str) -> Result<Box<dyn Project>, ChandlerError> {
    load_or_create_format(path, url, ProjectFormat::V3)
}
