use std::path::PathBuf;

use lazy_static::lazy_static;

use chandler::error::*;
use chandler::site::config::SitesConfig;

pub const CONFIG_DIR: &str = "chandler3";

lazy_static! {
    pub static ref DEFAULT_CONFIG_DIR_PATH: PathBuf = dirs::config_dir().unwrap().join(CONFIG_DIR);
    static ref DEFAULT_SITES_FILE_PATH: PathBuf = DEFAULT_CONFIG_DIR_PATH.join(SITES_FILENAME);
}

pub const SITES_FILENAME: &str = "sites.toml";

pub fn load_sites_config() -> Result<SitesConfig, ChandlerError> {
    if DEFAULT_SITES_FILE_PATH.exists() {
        Ok(SitesConfig::from_file(&*DEFAULT_SITES_FILE_PATH)?)
    } else {
        Ok(SitesConfig::load_default()?)
    }
}
