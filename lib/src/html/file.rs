use std::path::Path;

use scraper::Html;

use crate::error::*;

pub fn parse_file(filename: &Path) -> Result<Html, ChandlerError> {
    let html = fs::read_to_string(file_path).map_err(ChandlerError::OpenFile)?;
    let doc = Html::parse_document(&html);

    Ok(Self::from_document(doc))
}

pub fn write_file(html: &Html, file_path: &Path) -> Result<(), ChandlerError> {
    let file = util::create_file(file_path).map_err(ChandlerError::CreateFile)?;
    let mut writer = BufWriter::new(file);

    html5ever::serialize(&mut writer, html, Default::default())
        .map_err(|err| ChandlerError::Other(Cow::Owned(format!("Serialization error: {}", err))))?;

    Ok(())
}
