use std::path::PathBuf;

use crate::config::ResolvedCliConfig;

use lazy_static::lazy_static;
use regex::Regex;

use chandler::error::*;
use chandler::site::SiteResolver;

lazy_static! {
    static ref REGEX_SPLIT_URL: Regex = Regex::new(r#"^http(?:s)?://([\w\.]+)/(.+)/thread/(\d+)"#).unwrap();
}

pub fn generate_destination_path(
    cfg: &ResolvedCliConfig,
    site_resolver: &dyn SiteResolver,
    url: &str,
) -> Result<PathBuf, ChandlerError> {
    let site_info = site_resolver.resolve_site(url)?;

    if let Some(site_info) = site_info {
        Ok(cfg.save_to_path.join(site_info.name).join(site_info.path))
    } else {
        let cap = REGEX_SPLIT_URL
            .captures(url)
            .ok_or_else(|| ChandlerError::Other(format!("Invalid thread url: {}!", url).into()))?;

        let host = &cap[1];
        let board = &cap[2];
        let thread = &cap[3];

        let path = cfg.save_to_path.join("unknown").join(host).join(board).join(thread);

        Ok(path)
    }
}
