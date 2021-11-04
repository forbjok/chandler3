use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde_derive::{Deserialize, Serialize};

use chandler::error::*;
use chandler::util;

pub const CLI_CONFIG_FILENAME: &str = "cli.toml";

pub const DEFAULT_CLI_TOML: &str = include_str!("default_cli.toml");

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CliProgressBarStyle {
    Dot,
    Hash,
    Arrow,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CliProgressConfig {
    pub enable: bool,
    pub bar_style: CliProgressBarStyle,
}

impl Default for CliProgressConfig {
    fn default() -> Self {
        Self {
            enable: true,
            bar_style: CliProgressBarStyle::Dot,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CliConfig {
    pub progress: CliProgressConfig,
}

impl CliConfig {
    pub fn from_file(path: &Path) -> Result<Self, ChandlerError> {
        let mut file = util::open_file(path).map_err(ChandlerError::OpenFile)?;

        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str).map_err(ChandlerError::ReadFile)?;

        Self::from_str(&toml_str)
    }

    pub fn default_location() -> Option<PathBuf> {
        chandler::config::get_default_config_path()
    }

    fn path_from_location(path: &Path) -> Result<PathBuf, ChandlerError> {
        Ok(path.join(CLI_CONFIG_FILENAME))
    }

    pub fn from_location(path: &Path) -> Result<Self, ChandlerError> {
        let config_file_path = Self::path_from_location(path)?;

        if config_file_path.exists() {
            Ok(Self::from_file(&config_file_path)?)
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
                file.write_all(DEFAULT_CLI_TOML.as_bytes())
                    .map_err(ChandlerError::WriteFile)?;
            }
        }

        Ok(())
    }
}

impl FromStr for CliConfig {
    type Err = ChandlerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: Self = toml::from_str(s).map_err(|err| ChandlerError::ParseConfig(err.to_string().into()))?;

        Ok(config)
    }
}
