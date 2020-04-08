use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use dirs;
use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};

use chandler::util;

pub const CONFIG_DIR: &str = "chandler3";
pub const CONFIG_FILENAME: &str = "config.toml";

lazy_static! {
    static ref DEFAULT_CONFIG_DIR_PATH: PathBuf = dirs::config_dir().unwrap().join(CONFIG_DIR);
    static ref DEFAULT_CONFIG_FILE_PATH: PathBuf = DEFAULT_CONFIG_DIR_PATH.join(CONFIG_FILENAME);

    static ref DEFAULT_SAVE_TO_PATH: PathBuf = dirs::download_dir().unwrap().join("chandler3");
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CliConfig {
    pub save_to_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ResolvedCliConfig {
    pub save_to_path: PathBuf,
}

impl CliConfig {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let mut file = util::open_file(path).map_err(|err| err.to_string())?;

        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str).map_err(|err| err.to_string())?;

        Self::from_str(&toml_str)
    }

    pub fn from_default_location() -> Result<Self, String> {
        if !(*DEFAULT_CONFIG_FILE_PATH).exists() {
            return Ok(Self::default());
        }

        Self::from_file(&*DEFAULT_CONFIG_FILE_PATH)
    }

    pub fn resolve(self) -> Result<ResolvedCliConfig, String> {
        let save_to_path = self.save_to_path
            .map(|p| util::normalize_path(p))
            .unwrap_or_else(|| (*DEFAULT_SAVE_TO_PATH).clone());

        Ok(ResolvedCliConfig {
            save_to_path,
        })
    }
}

impl FromStr for CliConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: Self = toml::from_str(s).map_err(|err| err.to_string())?;

        Ok(config)
    }
}
