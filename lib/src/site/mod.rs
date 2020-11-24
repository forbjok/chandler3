use std::path::PathBuf;

use crate::error::*;
use crate::threadupdater::ParserType;

pub mod config;

pub struct SiteInfo {
    pub name: String,
    pub parser: ParserType,
    pub path: PathBuf,
}

pub trait SiteResolver {
    fn resolve_site(&self, url: &str) -> Result<Option<SiteInfo>, ChandlerError>;
}
