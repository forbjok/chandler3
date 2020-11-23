use std::path::Path;

pub mod common;
mod v2;
mod v3;

use crate::error::*;
use crate::ui::*;

pub use self::common::UpdateResult;

pub trait Project {
    fn update(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<UpdateResult, ChandlerError>;
    fn rebuild(&mut self, ui_handler: &mut dyn ChandlerUiHandler) -> Result<(), ChandlerError>;
    fn save(&self) -> Result<(), ChandlerError>;
}

pub trait ProjectLoader {
    type P: Project;

    fn create(path: &Path, url: &str) -> Result<Self::P, ChandlerError>;
    fn load(path: &Path) -> Result<Self::P, ChandlerError>;
    fn exists_at(path: &Path) -> bool;
}

pub fn exists_at(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();

    if v3::V3Project::exists_at(path) {
        true
    } else if v2::V2Project::exists_at(path) {
        true
    } else {
        false
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

pub fn load_or_create(path: impl AsRef<Path>, url: &str) -> Result<Box<dyn Project>, ChandlerError> {
    let path = path.as_ref();

    if exists_at(path) {
        load(path)
    } else {
        Ok(Box::new(v3::V3Project::create(path, url)?))
    }
}
