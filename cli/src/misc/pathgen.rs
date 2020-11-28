use crate::config::ResolvedCliConfig;

use lazy_static::lazy_static;
use regex::Regex;

use chandler::error::*;
use chandler::site::{SiteInfo, SiteResolver};
use chandler::threadupdater::ParserType;

lazy_static! {
    static ref REGEX_SPLIT_URL: Regex = Regex::new(r#"^http(?:s)?://([\w\.]+)/(?:(.+)/)*([^\.]+).*"#).unwrap();
}

pub fn generate_destination_path(
    cfg: &ResolvedCliConfig,
    site_resolver: &dyn SiteResolver,
    url: &str,
) -> Result<SiteInfo, ChandlerError> {
    let site_info = site_resolver.resolve_site(url)?;

    if let Some(mut site_info) = site_info {
        site_info.path = cfg.download_path.join(&site_info.name).join(&site_info.path);

        Ok(site_info)
    } else {
        let caps = REGEX_SPLIT_URL
            .captures(url)
            .ok_or_else(|| ChandlerError::Other(format!("Error parsing url: {}!", url).into()))?;

        let host = &caps[1];
        let board = &caps[2];
        let thread = &caps[3];

        let path = cfg.download_path.join("unknown").join(host).join(board).join(thread);

        Ok(SiteInfo {
            name: "unknown".to_owned(),
            parser: ParserType::Basic,
            path,
        })
    }
}
