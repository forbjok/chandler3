use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use serde_derive::Deserialize;

use crate::config::*;
use crate::error::*;
use crate::util;

use super::*;

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
    sites: HashMap<String, SiteDef>,
}

impl SitesConfig {
    pub fn from_file(path: &Path) -> Result<Self, ChandlerError> {
        use std::io::Read;

        let mut file = util::open_file(path).map_err(ChandlerError::OpenConfig)?;

        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str).map_err(ChandlerError::ReadConfig)?;

        Self::from_str(&toml_str)
    }

    pub fn load_default() -> Result<Self, ChandlerError> {
        DEFAULT_SITES_TOML.parse()
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
