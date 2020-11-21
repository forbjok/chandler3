use std::path::PathBuf;

use crate::config::ResolvedCliConfig;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref REGEX_SPLIT_URL: Regex = Regex::new(r#"^http(?:s)?://([\w\.]+)/(.+)/thread/(\d+)"#).unwrap();
}

pub fn generate_destination_path(cfg: &ResolvedCliConfig, url: &str) -> Result<PathBuf, String> {
    let cap = REGEX_SPLIT_URL
        .captures(url)
        .ok_or_else(|| format!("Invalid thread url: {}!", url))?;

    let host = &cap[1];
    let board = &cap[2];
    let thread = &cap[3];

    let path = cfg.save_to_path.join(host).join(board).join(thread);

    Ok(path)
}
