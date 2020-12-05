use std::path::PathBuf;

use log::error;

pub mod chandler;
mod parser;
mod regex;
pub mod sites;

pub use self::parser::*;
pub use self::regex::*;

pub const CONFIG_DIR: &str = "chandler3";

pub fn get_config_path() -> Option<PathBuf> {
    let config_path = dirs::config_dir().map(|p| p.join(CONFIG_DIR));

    if config_path.is_none() {
        error!("Could not get configuration path!");
    }

    config_path
}
