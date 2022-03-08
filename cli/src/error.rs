use std::borrow::Cow;

use chandler::{ChandlerError, DownloadError};

#[derive(Debug)]
pub enum CliErrorKind {
    //Arguments,
    Config,
    Other,
}

impl CliErrorKind {
    pub fn exit_code(&self) -> i32 {
        match self {
            //Self::Arguments => 1,
            Self::Config => 2,
            Self::Other => 101,
        }
    }
}

#[derive(Debug)]
pub struct CliError {
    pub kind: CliErrorKind,
    pub description: Cow<'static, str>,
}

impl CliError {
    pub fn new<S: Into<Cow<'static, str>>>(kind: CliErrorKind, description: S) -> CliError {
        CliError {
            kind,
            description: description.into(),
        }
    }
}

impl From<ChandlerError> for CliError {
    fn from(error: ChandlerError) -> Self {
        match error {
            ChandlerError::CreateProject(err) => {
                CliError::new(CliErrorKind::Config, format!("Error creating project: {err}"))
            }
            ChandlerError::LoadProject(err) => {
                CliError::new(CliErrorKind::Config, format!("Error loading project: {err}"))
            }
            ChandlerError::OpenConfig(err) => {
                CliError::new(CliErrorKind::Config, format!("Error opening config file: {err}"))
            }
            ChandlerError::ReadConfig(err) => {
                CliError::new(CliErrorKind::Config, format!("Error reading config file: {err}"))
            }
            ChandlerError::ParseConfig(err) => {
                CliError::new(CliErrorKind::Config, format!("Error parsing configuration: {err}"))
            }
            ChandlerError::Config(err) => CliError::new(CliErrorKind::Config, format!("Configuration error: {err}")),
            ChandlerError::OpenFile(err) => CliError::new(CliErrorKind::Config, format!("Error opening file: {err}")),
            ChandlerError::CreateFile(err) => {
                CliError::new(CliErrorKind::Config, format!("Error creating file: {err}"))
            }
            ChandlerError::ReadFile(err) => CliError::new(CliErrorKind::Config, format!("Error reading file: {err}")),
            ChandlerError::WriteFile(err) => CliError::new(CliErrorKind::Config, format!("Error writing file: {err}")),
            ChandlerError::Download(err) => match err {
                DownloadError::Http { code, description } => {
                    CliError::new(CliErrorKind::Other, format!("HTTP error: {code} {description}"))
                }
                DownloadError::Network(err) => CliError::new(CliErrorKind::Other, err.to_string()),
                DownloadError::Other(err) => CliError::new(CliErrorKind::Other, err.to_string()),
            },
            ChandlerError::Other(err) => CliError::new(CliErrorKind::Other, err.to_string()),
        }
    }
}
