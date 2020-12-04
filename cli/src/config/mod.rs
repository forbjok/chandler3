use std::io::Read;
use std::path::Path;
use std::str::FromStr;

use serde_derive::{Deserialize, Serialize};

use chandler::error::*;
use chandler::util;

pub const CONFIG_DIR: &str = "chandler3";
pub const CLI_CONFIG_FILENAME: &str = "cli.toml";

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

    pub fn from_default_location() -> Result<Self, ChandlerError> {
        let config_path = if let Some(config_path) = dirs::config_dir() {
            config_path
        } else {
            ".".into()
        };

        let config_path = config_path.join(CONFIG_DIR);

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let config_file_path = config_path.join(CLI_CONFIG_FILENAME);

        Self::from_file(&config_file_path)
    }
}

impl FromStr for CliConfig {
    type Err = ChandlerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: Self = toml::from_str(s).map_err(|err| ChandlerError::ParseConfig(err.to_string().into()))?;

        Ok(config)
    }
}
