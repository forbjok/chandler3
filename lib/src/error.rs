use std::borrow::Cow;
use std::io;

use thiserror::Error;

use crate::util;

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
    OpenConfig(util::FileError),
    #[error("Error reading config")]
    ReadConfig(io::Error),
    #[error("Error parsing config")]
    ParseConfig(Cow<'static, str>),
    #[error("Configuration error")]
    Config(Cow<'static, str>),
    #[error("Error opening file")]
    OpenFile(util::FileError),
    #[error("Error creating file")]
    CreateFile(util::FileError),
    #[error("Error reading file")]
    ReadFile(io::Error),
    #[error("Error writing file")]
    WriteFile(io::Error),
    #[error("Download error")]
    Download(DownloadError),
    #[error("Error")]
    Other(Cow<'static, str>),
}
