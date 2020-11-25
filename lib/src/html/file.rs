use std::path::Path;

use crate::error::*;
use crate::util;

pub fn parse_file(filename: &Path) -> Result<kuchiki::NodeRef, ChandlerError> {
    use html5ever::tendril::TendrilSink;

    let mut f = util::open_file(filename).map_err(ChandlerError::OpenFile)?;

    let dom = kuchiki::parse_html()
        .from_utf8()
        .read_from(&mut f)
        .map_err(ChandlerError::ReadFile)?;

    Ok(dom)
}
