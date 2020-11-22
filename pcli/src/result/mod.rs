use serde_derive::Serialize;

use chandler::ChandlerError;

mod update;

pub use self::update::*;

#[derive(Debug, Serialize)]
pub struct PcliError {
    pub code: u32,
    pub description: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "result", content = "data")]
#[serde(rename_all = "lowercase")]
pub enum PcliResult<T> {
    Success(T),
    Error(PcliError),
}

impl From<ChandlerError> for PcliError {
    fn from(error: ChandlerError) -> Self {
        match error {
            ChandlerError::CreateProject(err) => PcliError {
                code: 10001,
                description: format!("Error creating project: {}", err.to_string()),
            },
            ChandlerError::LoadProject(err) => PcliError {
                code: 10002,
                description: format!("Error loading project: {}", err.to_string()),
            },
            ChandlerError::OpenConfig(err) => PcliError {
                code: 10003,
                description: format!("Error opening config file: {}", err.to_string()),
            },
            ChandlerError::ReadConfig(err) => PcliError {
                code: 10004,
                description: format!("Error reading config file: {}", err),
            },
            ChandlerError::ParseConfig(err) => PcliError {
                code: 10005,
                description: format!("Error parsing configuration: {}", err),
            },
            ChandlerError::Config(err) => PcliError {
                code: 10006,
                description: format!("Configuration error: {}", err),
            },
            ChandlerError::OpenFile(err) => PcliError {
                code: 10007,
                description: format!("Error opening file: {}", err.to_string()),
            },
            ChandlerError::CreateFile(err) => PcliError {
                code: 10008,
                description: format!("Error creating file: {}", err.to_string()),
            },
            ChandlerError::ReadFile(err) => PcliError {
                code: 10009,
                description: format!("Error reading file: {}", err),
            },
            ChandlerError::WriteFile(err) => PcliError {
                code: 10010,
                description: format!("Error writing file: {}", err),
            },
            ChandlerError::Download(err) => PcliError {
                code: 10011,
                description: err.to_string(),
            },
            ChandlerError::DownloadHttpStatus {
                status_code,
                description,
            } => PcliError {
                code: 10012,
                description: format!("Download HTTP error: {} {}", status_code, description),
            },
            ChandlerError::Other(err) => PcliError {
                code: 10000,
                description: err.to_string(),
            },
        }
    }
}

impl<TS: Into<TD>, TD> From<Result<TS, PcliError>> for PcliResult<TD> {
    fn from(result: Result<TS, PcliError>) -> Self {
        match result {
            Ok(v) => PcliResult::Success(v.into()),
            Err(err) => PcliResult::Error(err),
        }
    }
}
