use std::collections::HashSet;

use serde_derive::{Deserialize, Serialize};

use crate::error::*;
use crate::threadupdater::ParserType;
use crate::util;

use super::*;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Parser {
    #[serde(rename = "4chan")]
    FourChan,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub parser: Parser,
    pub url: String,
    pub download_extensions: HashSet<String>,
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

impl From<Parser> for ParserType {
    fn from(parser: Parser) -> Self {
        match parser {
            Parser::FourChan => ParserType::FourChan,
        }
    }
}

impl From<ParserType> for Parser {
    fn from(parser: ParserType) -> Self {
        match parser {
            ParserType::FourChan => Parser::FourChan,
        }
    }
}
