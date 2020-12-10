use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde_derive::Deserialize;

use crate::error::*;
use crate::misc::site_resolver::{SiteInfo, SiteResolver};
use crate::util;

use super::*;

pub const SITES_CONFIG_FILENAME: &str = "sites.toml";

pub const BUILTIN_SITES_TOML: &str = include_str!("builtin_sites.toml");
pub const DEFAULT_SITES_TOML: &str = include_str!("default_sites.toml");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SiteDef {
    pub url_regexes: Regexes,
    pub parser: Parser,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SitesConfig {
    #[serde(default = "default_include_builtin_sites")]
    pub include_builtin_sites: bool,
    pub sites: HashMap<String, SiteDef>,
}

/// Used to specify serde default value for the "include_builtin_sites" field.
fn default_include_builtin_sites() -> bool {
    true
}

impl SitesConfig {
    pub fn from_file(path: &Path) -> Result<Self, ChandlerError> {
        use std::io::Read;

        let mut file = util::open_file(path).map_err(ChandlerError::OpenConfig)?;

        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str).map_err(ChandlerError::ReadConfig)?;

        Self::from_str(&toml_str)
    }

    pub fn default_location() -> Option<PathBuf> {
        get_default_config_path()
    }

    pub fn from_location(path: &Path) -> Result<Self, ChandlerError> {
        let config_file_path = path.join(SITES_CONFIG_FILENAME);

        if config_file_path.exists() {
            let mut user_config = SitesConfig::from_file(&config_file_path)?;
            if user_config.include_builtin_sites {
                user_config.merge_from(SitesConfig::load_builtin()?)
            }

            Ok(user_config)
        } else {
            Ok(SitesConfig::load_builtin()?)
        }
    }

    pub fn from_default_location() -> Result<Self, ChandlerError> {
        if let Some(path) = Self::default_location() {
            Self::from_location(&path)
        } else {
            Self::load_builtin()
        }
    }

    pub fn load_builtin() -> Result<Self, ChandlerError> {
        BUILTIN_SITES_TOML.parse()
    }

    pub fn merge_from(&mut self, other: Self) {
        self.sites.extend(other.sites);
    }

    pub fn write_default() -> Result<(), ChandlerError> {
        if let Some(config_file_path) = Self::default_location() {
            if !config_file_path.exists() {
                // Create config directory if necessary.
                util::create_parent_dir(&config_file_path)
                    .map_err(|err| ChandlerError::Other(err.to_string().into()))?;

                // Write config file.
                let mut file = util::create_file(config_file_path).map_err(ChandlerError::CreateFile)?;
                file.write_all(DEFAULT_SITES_TOML.as_bytes())
                    .map_err(ChandlerError::WriteFile)?;
            }
        }

        Ok(())
    }
}

impl FromStr for SitesConfig {
    type Err = ChandlerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: Self = toml::from_str(s).map_err(|err| ChandlerError::ParseConfig(Cow::Owned(err.to_string())))?;

        Ok(config)
    }
}

impl SiteResolver for SitesConfig {
    fn resolve_site(&self, url: &str) -> Result<Option<SiteInfo>, ChandlerError> {
        for (name, def) in self.sites.iter() {
            let regexes = def.url_regexes.build_regexes()?;

            for regex in regexes.iter() {
                if let Some(caps) = regex.captures(url) {
                    let mut path = PathBuf::new();

                    for c in caps.iter().skip(1) {
                        if let Some(m) = c {
                            path.push(m.as_str());
                        }
                    }

                    return Ok(Some(SiteInfo {
                        name: name.clone(),
                        parser: def.parser.into(),
                        path,
                    }));
                }
            }
        }

        Ok(None)
    }
}
