use std::path::Path;

use crate::error::ChandlerError;
use crate::html;

pub mod fourchan;

pub trait CreateThreadUpdater {
    fn create_thread_updater_from(&self, path: &Path) -> Result<Box<dyn ThreadUpdater>, ChandlerError>;
}

pub trait ThreadUpdater {
    fn perform_initial_cleanup(&mut self) -> Result<UpdateResult, ChandlerError>;
    fn update_from(&mut self, path: &Path) -> Result<UpdateResult, ChandlerError>;
    fn write_file(&self, file_path: &Path) -> Result<(), ChandlerError>;
}

pub struct UpdateResult {
    pub new_links: Vec<html::Link>,
}
