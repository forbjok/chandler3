use serde_derive::{Deserialize, Serialize};
use serde_json;

use crate::error::*;
use crate::threadupdater::CreateThreadUpdater;
use crate::util;

use super::*;

#[derive(Debug, Deserialize, Serialize)]
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
    pub download_extensions: Vec<String>,
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

impl CreateThreadUpdater for Parser {
    fn create_thread_updater_from(&self, path: &Path) -> Result<Box<dyn ThreadUpdater>, ChandlerError> {
        use crate::threadupdater::*;

        Ok(match self {
            Self::FourChan => Box::new(fourchan::FourchanThreadUpdater::from_file(path)?),
        })
    }
}
