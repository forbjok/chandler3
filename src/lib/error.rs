use std::borrow::Cow;
use std::fmt;
use std::io;

use crate::util;

#[derive(Debug)]
pub enum ChandlerError {
    CreateProject(Cow<'static, str>),
    LoadProject(Cow<'static, str>),
    OpenConfig(util::FileError),
    ReadConfig(io::Error),
    ParseConfig(Cow<'static, str>),
    Config(Cow<'static, str>),
    OpenFile(util::FileError),
    CreateFile(util::FileError),
    ReadFile(io::Error),
    WriteFile(io::Error),
    Other(Cow<'static, str>),
}

impl fmt::Display for ChandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CreateProject(err) => write!(f, "{}", err),
            Self::LoadProject(err) => write!(f, "{}", err),
            Self::OpenConfig(err) => write!(f, "{}", err),
            Self::ReadConfig(err) => write!(f, "{}", err),
            Self::ParseConfig(err) => write!(f, "{}", err),
            Self::Config(err) => write!(f, "{}", err),
            Self::OpenFile(err) => write!(f, "{}", err),
            Self::CreateFile(err) => write!(f, "{}", err),
            Self::ReadFile(err) => write!(f, "{}", err),
            Self::WriteFile(err) => write!(f, "{}", err),
            Self::Other(err) => write!(f, "{}", err),
        }
    }
}
