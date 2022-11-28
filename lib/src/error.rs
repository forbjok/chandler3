use std::borrow::Cow;
use std::io;

use thiserror::Error;

#[derive(Debug)]
pub enum DownloadError {
    Http { code: u16, description: Cow<'static, str> },
    Network(Cow<'static, str>),
    Other(Cow<'static, str>),
}

#[derive(Debug, Error)]
pub enum ChandlerError {
    #[error("Error creating project")]
    CreateProject(Cow<'static, str>),
    #[error("Error loading project")]
    LoadProject(Cow<'static, str>),
    #[error("Error opening config")]
    OpenConfig(anyhow::Error),
    #[error("Error reading config")]
    ReadConfig(io::Error),
    #[error("Error parsing config")]
    ParseConfig(Cow<'static, str>),
    #[error("Configuration error")]
    Config(Cow<'static, str>),
    #[error("Error opening file")]
    OpenFile(anyhow::Error),
    #[error("Error creating file")]
    CreateFile(anyhow::Error),
    #[error("Error reading file")]
    ReadFile(io::Error),
    #[error("Error writing file")]
    WriteFile(io::Error),
    #[error("Download error")]
    Download(DownloadError),
    #[error("Error")]
    Other(Cow<'static, str>),
}
