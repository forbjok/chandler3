use std::borrow::Cow;

use scraper::Html;

use super::*;
use crate::error::ChandlerError;
use crate::html;
use crate::util;

pub struct BasicThread {
    pub html: Html,
}

impl HtmlDocument for BasicThread {
    type Document = Html;

    fn from_document(document: Self::Document) -> Self {
        Self { html: document }
    }

    fn into_document(self) -> Self::Document {
        self.html
    }

    fn from_file(file_path: &Path) -> Result<Self, ChandlerError> {
        let node = html::parse_file(file_path)?;

        Ok(Self::from_document(node))
    }

    fn write_file(&self, file_path: &Path) -> Result<(), ChandlerError> {
        html::write_file(&self.html, file_path)
    }

    fn for_links(&self, mut action: impl FnMut(html::Link) -> Result<(), ChandlerError>) -> Result<(), ChandlerError> {
        let links = html::find_links(self.root.clone());

        for link in links.into_iter() {
            action(link)?;
        }

        Ok(())
    }

    fn purge_scripts(&mut self) -> Result<(), ChandlerError> {
        html::purge_scripts(&mut self.html);

        Ok(())
    }
}
