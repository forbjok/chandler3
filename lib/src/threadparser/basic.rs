use scraper::Html;

use super::*;
use crate::error::ChandlerError;
use crate::html;

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

    fn for_links_mut(
        &mut self,
        mut action: impl FnMut(html::LinkMut) -> Result<(), ChandlerError>,
    ) -> Result<(), ChandlerError> {
        html::for_links_mut(&mut self.html, None, action)
    }

    fn purge_scripts(&mut self) -> Result<(), ChandlerError> {
        html::purge_scripts(&mut self.html);

        Ok(())
    }
}
