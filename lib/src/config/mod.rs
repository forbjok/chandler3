use std::path::PathBuf;

use lazy_static::lazy_static;

pub mod chandler;
mod parser;
mod regex;
pub mod sites;

pub use self::parser::*;
pub use self::regex::*;

pub const CONFIG_DIR: &str = "chandler3";

lazy_static! {
    static ref DEFAULT_CONFIG_DIR_PATH: PathBuf = dirs::config_dir().unwrap().join(CONFIG_DIR);
}
