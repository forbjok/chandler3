use std::borrow::Cow;
use std::path::PathBuf;

use lazy_static::lazy_static;
use regex::Regex;

use crate::error::*;
use crate::threadupdater::ParserType;

lazy_static! {
    static ref REGEX_SPLIT_URL: Regex = Regex::new(r#"^http(?:s)?://([\w\.:]+)/(?:(.+)/)*([^\.]+).*"#).unwrap();
}

pub struct SiteInfo {
    pub name: String,
    pub parser: ParserType,
    pub path: PathBuf,
}

pub trait SiteResolver {
    fn resolve_site(&self, url: &str) -> Result<Option<SiteInfo>, ChandlerError>;
}

pub fn unknown_site(url: &str) -> Result<SiteInfo, ChandlerError> {
    let caps = REGEX_SPLIT_URL
        .captures(url)
        .ok_or_else(|| ChandlerError::Other(format!("Error parsing url: {}!", url).into()))?;

    let host = &caps[1];
    let board = &caps[2];
    let thread = &caps[3];

    let mut path = PathBuf::new();
    path.push(sanitize_path(host).as_ref());
    path.push(sanitize_path(board).as_ref());
    path.push(sanitize_path(thread).as_ref());

    Ok(SiteInfo {
        name: "unknown".to_owned(),
        parser: ParserType::Basic,
        path,
    })
}

/// Sanitize path to ensure it does not contain invalid filesystem characters.
pub fn sanitize_path(s: &str) -> Cow<str> {
    lazy_static! {
        static ref SANITIZE_PATH_REGEX: Regex = Regex::new(r#":|\*|\|"#).unwrap();
    }

    SANITIZE_PATH_REGEX.replace_all(s, "_")
}
