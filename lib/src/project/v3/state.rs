use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

use crate::error::*;
use crate::util;

use super::*;

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
pub struct ProjectState {
    pub last_modified: Option<DateTime<Utc>>,
    pub is_dead: bool,
    pub links: Links,
}

impl ProjectState {
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
