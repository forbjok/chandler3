use std::path::Path;

use crate::error::*;
use crate::util;

pub fn parse_file(filename: &Path) -> Result<kuchiki::NodeRef, ChandlerError> {
    use html5ever::tendril::TendrilSink;

    let mut f = util::open_file(filename).map_err(|err| ChandlerError::OpenFile(err))?;

    let dom = kuchiki::parse_html()
        .from_utf8()
        .read_from(&mut f)
        .map_err(|err| ChandlerError::ReadFile(err))?;

    Ok(dom)
}
