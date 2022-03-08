use std::collections::BTreeSet;

use serde_derive::{Deserialize, Serialize};

pub use crate::config::Parser;
use crate::error::*;
use crate::util;

use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub parser: Parser,
    pub url: String,
    pub download_extensions: BTreeSet<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    pub failed: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub last_modified: Option<DateTime<Utc>>,
    pub is_dead: bool,
    pub links: Links,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ChandlerError> {
        let file = util::open_file(path).map_err(ChandlerError::OpenConfig)?;

        serde_json::from_reader(file).map_err(|err| ChandlerError::ParseConfig(Cow::Owned(err.to_string())))
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ChandlerError> {
        let file = util::create_file(path).map_err(ChandlerError::CreateFile)?;

        serde_json::to_writer_pretty(file, self)
            .map_err(|err| ChandlerError::ParseConfig(Cow::Owned(err.to_string())))?;

        Ok(())
    }
}

impl State {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ChandlerError> {
        let file = util::open_file(path).map_err(ChandlerError::OpenConfig)?;

        serde_json::from_reader(file).map_err(|err| ChandlerError::ParseConfig(Cow::Owned(err.to_string())))
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ChandlerError> {
        let file = util::create_file(path).map_err(ChandlerError::CreateFile)?;

        serde_json::to_writer_pretty(file, self)
            .map_err(|err| ChandlerError::ParseConfig(Cow::Owned(err.to_string())))?;

        Ok(())
    }
}

impl From<&ProjectState> for Config {
    fn from(state: &ProjectState) -> Self {
        Self {
            parser: state.parser.into(),
            url: state.thread_url.clone(),
            download_extensions: state.download_extensions.clone(),
        }
    }
}

impl From<&ProjectState> for State {
    fn from(state: &ProjectState) -> Self {
        Self {
            last_modified: state.last_modified,
            is_dead: state.is_dead,
            links: Links {
                failed: state
                    .new_links
                    .iter()
                    .chain(state.failed_links.iter())
                    .map(|l| l.url.clone())
                    .collect(),
            },
        }
    }
}
