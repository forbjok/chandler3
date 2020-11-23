use std::collections::HashSet;

use serde_derive::{Deserialize, Serialize};
use serde_json;

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
pub struct ProjectConfig {
    pub parser: Parser,
    pub url: String,
    pub download_extensions: HashSet<String>,
}

impl ProjectConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ChandlerError> {
        let file = util::open_file(path).map_err(|err| ChandlerError::OpenConfig(err))?;

        Ok(serde_json::from_reader(file).map_err(|err| ChandlerError::ParseConfig(Cow::Owned(err.to_string())))?)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ChandlerError> {
        let file = util::create_file(path).map_err(|err| ChandlerError::CreateFile(err))?;

        serde_json::to_writer_pretty(file, self)
            .map_err(|err| ChandlerError::ParseConfig(Cow::Owned(err.to_string())))?;

        Ok(())
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
