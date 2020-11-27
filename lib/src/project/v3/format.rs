use std::collections::BTreeSet;

use serde_derive::{Deserialize, Serialize};

pub use crate::config::Parser;
use crate::error::*;
use crate::util;

use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub parser: Parser,
    pub url: String,
    pub download_extensions: BTreeSet<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Link {
    pub url: String,
    pub path: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Links {
    pub new: Vec<Link>,
    pub failed: Vec<Link>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct State {
    pub last_modified: Option<DateTime<Utc>>,
    pub is_dead: bool,
    pub links: Links,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ChandlerError> {
        let file = util::open_file(path).map_err(ChandlerError::OpenConfig)?;

        Ok(serde_json::from_reader(file).map_err(|err| ChandlerError::ParseConfig(Cow::Owned(err.to_string())))?)
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

        Ok(serde_json::from_reader(file).map_err(|err| ChandlerError::ParseConfig(Cow::Owned(err.to_string())))?)
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
                new: state
                    .new_links
                    .iter()
                    .map(|l| Link {
                        url: l.url.clone(),
                        path: l.path.clone(),
                    })
                    .collect(),
                failed: state
                    .failed_links
                    .iter()
                    .map(|l| Link {
                        url: l.url.clone(),
                        path: l.path.clone(),
                    })
                    .collect(),
            },
        }
    }
}
