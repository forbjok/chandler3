use std::borrow::Cow;

mod grab;

pub use grab::*;

use chandler::{ChandlerError, DownloadError};

#[derive(Debug)]
pub enum CommandErrorKind {
    //Arguments,
    Config,
    Other,
    Network,
}

impl CommandErrorKind {
    pub fn exit_code(&self) -> i32 {
        match self {
            //Self::Arguments => 1,
            Self::Config => 2,
            Self::Other => 101,
            Self::Network => 102,
        }
    }
}

#[derive(Debug)]
pub struct CommandError {
    pub kind: CommandErrorKind,
    pub description: Cow<'static, str>,
}

impl CommandError {
    pub fn new<S: Into<Cow<'static, str>>>(kind: CommandErrorKind, description: S) -> CommandError {
        CommandError {
            kind,
            description: description.into(),
        }
    }
}

impl From<ChandlerError> for CommandError {
    fn from(error: ChandlerError) -> Self {
        match error {
            ChandlerError::CreateProject(err) => {
                CommandError::new(CommandErrorKind::Config, format!("Error creating project: {err}"))
            }
            ChandlerError::LoadProject(err) => {
                CommandError::new(CommandErrorKind::Config, format!("Error loading project: {err}"))
            }
            ChandlerError::OpenConfig(err) => {
                CommandError::new(CommandErrorKind::Config, format!("Error opening config file: {err}"))
            }
            ChandlerError::ReadConfig(err) => {
                CommandError::new(CommandErrorKind::Config, format!("Error reading config file: {err}"))
            }
            ChandlerError::ParseConfig(err) => {
                CommandError::new(CommandErrorKind::Config, format!("Error parsing configuration: {err}"))
            }
            ChandlerError::Config(err) => {
                CommandError::new(CommandErrorKind::Config, format!("Configuration error: {err}"))
            }
            ChandlerError::OpenFile(err) => {
                CommandError::new(CommandErrorKind::Config, format!("Error opening file: {err}"))
            }
            ChandlerError::CreateFile(err) => {
                CommandError::new(CommandErrorKind::Config, format!("Error creating file: {err}"))
            }
            ChandlerError::ReadFile(err) => {
                CommandError::new(CommandErrorKind::Config, format!("Error reading file: {err}"))
            }
            ChandlerError::WriteFile(err) => {
                CommandError::new(CommandErrorKind::Config, format!("Error writing file: {err}"))
            }
            ChandlerError::Download(err) => match err {
                DownloadError::Http { code, description } => CommandError::new(
                    CommandErrorKind::Other,
                    format!("Download HTTP error: {code} {description}"),
                ),
                DownloadError::Network(err) => CommandError::new(CommandErrorKind::Network, err.to_string()),
                DownloadError::Other(err) => CommandError::new(CommandErrorKind::Other, err.to_string()),
            },
            ChandlerError::Other(err) => CommandError::new(CommandErrorKind::Other, err.to_string()),
        }
    }
}
