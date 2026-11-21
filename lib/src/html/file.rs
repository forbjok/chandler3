use std::{borrow::Cow, fs, io::BufWriter, path::Path};

use anyhow::Context as _;
use scraper::Html;

use crate::{error::*, util};

pub fn parse_file(file_path: &Path) -> Result<Html, ChandlerError> {
    let html = fs::read_to_string(file_path)
        .context("Error reading HTML file")
        .map_err(ChandlerError::OpenFile)?;

    Ok(Html::parse_document(&html))
}

pub fn write_file(html: &Html, file_path: &Path) -> Result<(), ChandlerError> {
    let file = util::create_file(file_path).map_err(ChandlerError::CreateFile)?;
    let mut writer = BufWriter::new(file);

    html5ever::serialize(&mut writer, html, Default::default())
        .map_err(|err| ChandlerError::Other(Cow::Owned(format!("Serialization error: {}", err))))?;

    Ok(())
}
