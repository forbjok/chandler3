use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};

use chandler::util;
use cli_common::config::DEFAULT_CONFIG_DIR_PATH;

pub const CONFIG_FILENAME: &str = "config.toml";

lazy_static! {
    static ref DEFAULT_CONFIG_FILE_PATH: PathBuf = DEFAULT_CONFIG_DIR_PATH.join(CONFIG_FILENAME);
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CliConfig {
    pub download_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ResolvedCliConfig {
    pub download_path: PathBuf,
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
        let download_path = if let Some(download_path) = self.download_path {
            util::normalize_path(download_path)
        } else if let Some(os_download_path) = dirs::download_dir() {
            os_download_path.join("chandler3")
        } else {
            return Err(
                "No default download directory found. A download path must be specified in the Chandler config file."
                    .to_owned(),
            );
        };

        Ok(ResolvedCliConfig { download_path })
    }
}

impl FromStr for CliConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: Self = toml::from_str(s).map_err(|err| err.to_string())?;

        Ok(config)
    }
}
