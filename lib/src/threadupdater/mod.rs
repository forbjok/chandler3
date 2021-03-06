use std::path::Path;

use crate::error::ChandlerError;
use crate::html;
use crate::threadparser::aspnetchan::AspNetChanThread;
use crate::threadparser::foolfuuka::FoolFuukaThread;
use crate::threadparser::fourchan::FourchanThread;
use crate::threadparser::kusabax::KusabaxThread;
use crate::threadparser::lainchan::LainchanThread;
use crate::threadparser::ponychan::PonychanThread;
use crate::threadparser::tinyboard::TinyboardThread;

mod basic;
mod merging;

pub use self::basic::*;
pub use self::merging::*;

#[derive(Clone, Copy, Debug)]
pub enum ParserType {
    Basic,
    FourChan,
    Tinyboard,
    AspNetChan,
    Kusabax,
    FoolFuuka,
    Ponychan,
    Lainchan,
}

pub trait CreateThreadUpdater {
    fn create_thread_updater_from(&self, path: &Path) -> Result<Box<dyn ThreadUpdater>, ChandlerError>;
}

pub trait ThreadUpdater {
    fn perform_initial_cleanup(&mut self) -> Result<UpdateResult, ChandlerError>;
    fn update_from(&mut self, path: &Path) -> Result<UpdateResult, ChandlerError>;
    fn write_file(&self, file_path: &Path) -> Result<(), ChandlerError>;
}

#[derive(Debug)]
pub struct UpdateResult {
    pub is_archived: bool,
    pub new_post_count: u32,
    pub new_links: Vec<html::Link>,
}

impl CreateThreadUpdater for ParserType {
    fn create_thread_updater_from(&self, path: &Path) -> Result<Box<dyn ThreadUpdater>, ChandlerError> {
        Ok(match self {
            Self::Basic => Box::new(BasicThreadUpdater::from_file(path)?),
            Self::FourChan => Box::new(MergingThreadUpdater::<FourchanThread>::from_file(path)?),
            Self::Tinyboard => Box::new(MergingThreadUpdater::<TinyboardThread>::from_file(path)?),
            Self::AspNetChan => Box::new(MergingThreadUpdater::<AspNetChanThread>::from_file(path)?),
            Self::Kusabax => Box::new(MergingThreadUpdater::<KusabaxThread>::from_file(path)?),
            Self::FoolFuuka => Box::new(MergingThreadUpdater::<FoolFuukaThread>::from_file(path)?),
            Self::Ponychan => Box::new(MergingThreadUpdater::<PonychanThread>::from_file(path)?),
            Self::Lainchan => Box::new(MergingThreadUpdater::<LainchanThread>::from_file(path)?),
        })
    }
}
