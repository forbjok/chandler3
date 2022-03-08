use std::path::PathBuf;

pub mod chandler;
mod parser;
mod regex;
pub mod sites;

use tracing::error;

use crate::error::*;

pub use self::parser::*;
pub use self::regex::*;

pub const CONFIG_DIR: &str = "chandler3";

pub fn get_default_config_path() -> Option<PathBuf> {
    let config_path = dirs::config_dir().map(|p| p.join(CONFIG_DIR));

    if config_path.is_none() {
        error!("Could not get configuration path!");
    }

    config_path
}

pub fn generate_default_config() -> Result<(), ChandlerError> {
    chandler::ChandlerConfig::write_default()?;
    sites::SitesConfig::write_default()?;

    Ok(())
}
