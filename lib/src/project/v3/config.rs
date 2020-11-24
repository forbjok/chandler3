use std::collections::HashSet;

use serde_derive::{Deserialize, Serialize};
use serde_json;

pub use crate::config::Parser;
use crate::error::*;
use crate::util;

use super::*;

#[derive(Debug, Deserialize, Serialize)]
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
