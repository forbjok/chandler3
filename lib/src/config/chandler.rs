use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde_derive::{Deserialize, Serialize};

use crate::error::*;
use crate::util;

use super::*;

pub const CONFIG_FILENAME: &str = "config.toml";

pub const DEFAULT_CONFIG_TOML: &str = include_str!("default_config.toml");

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ChandlerConfig {
    pub download_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ResolvedChandlerConfig {
    pub download_path: PathBuf,
}

impl ChandlerConfig {
    pub fn from_file(path: &Path) -> Result<Self, ChandlerError> {
        let mut file = util::open_file(path).map_err(ChandlerError::OpenFile)?;

        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str).map_err(ChandlerError::ReadFile)?;

        Self::from_str(&toml_str)
    }

    pub fn default_location() -> Option<PathBuf> {
        get_default_config_path()
    }

    fn path_from_location(path: &Path) -> Result<PathBuf, ChandlerError> {
        Ok(path.join(CONFIG_FILENAME))
    }

    pub fn from_location(path: &Path) -> Result<Self, ChandlerError> {
        let config_file_path = Self::path_from_location(path)?;

        if config_file_path.exists() {
            Ok(Self::from_file(&config_file_path)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn from_default_location() -> Result<Self, ChandlerError> {
        if let Some(path) = Self::default_location() {
            Self::from_location(&path)
        } else {
            Ok(Self::default())
        }
    }

    pub fn write_default() -> Result<(), ChandlerError> {
        if let Some(config_location) = Self::default_location() {
            let config_file_path = Self::path_from_location(&config_location)?;

            if !config_file_path.exists() {
                // Create config directory if necessary.
                util::create_parent_dir(&config_file_path)
                    .map_err(|err| ChandlerError::Other(err.to_string().into()))?;

                // Write config file.
                let mut file = util::create_file(config_file_path).map_err(ChandlerError::CreateFile)?;
                file.write_all(DEFAULT_CONFIG_TOML.as_bytes())
                    .map_err(ChandlerError::WriteFile)?;
            }
        }

        Ok(())
    }

    pub fn resolve(self) -> Result<ResolvedChandlerConfig, ChandlerError> {
        let download_path = if let Some(download_path) = self.download_path {
            util::normalize_path(download_path)
        } else if let Some(os_download_path) = dirs::download_dir() {
            os_download_path.join("chandler3")
        } else {
            return Err(ChandlerError::Config(
                "No default download directory found. A download path must be specified in the Chandler config file."
                    .into(),
            ));
        };

        Ok(ResolvedChandlerConfig { download_path })
    }
}

impl FromStr for ChandlerConfig {
    type Err = ChandlerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: Self = toml::from_str(s).map_err(|err| ChandlerError::ParseConfig(err.to_string().into()))?;

        Ok(config)
    }
}
