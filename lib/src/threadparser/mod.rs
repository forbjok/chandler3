use std::path::Path;

use crate::error::*;
use crate::html;

pub mod aspnetchan;
pub mod basic;
pub mod foolfuuka;
pub mod fourchan;
pub mod kusabax;
pub mod tinyboard;

pub trait HtmlDocument: Sized {
    type Document;

    fn from_document(document: Self::Document) -> Self;
    fn into_document(self) -> Self::Document;

    fn from_file(file_path: &Path) -> Result<Self, ChandlerError>;
    fn write_file(&self, file_path: &Path) -> Result<(), ChandlerError>;

    fn for_links(&self, action: impl FnMut(html::Link) -> Result<(), ChandlerError>) -> Result<(), ChandlerError>;

    /// Purge all script tags
    fn purge_scripts(&self) -> Result<(), ChandlerError>;
}

pub trait MergeableImageboardThread: HtmlDocument {
    type Reply;

    fn get_all_replies(&self) -> Result<Box<dyn Iterator<Item = Self::Reply>>, ChandlerError>;

    fn merge_replies_from(&mut self, new: Self) -> Result<Vec<Self::Reply>, ChandlerError>;

    fn for_reply_links(
        &self,
        reply: &Self::Reply,
        action: impl FnMut(html::Link) -> Result<(), ChandlerError>,
    ) -> Result<(), ChandlerError>;

    fn is_archived(&self) -> Result<bool, ChandlerError>;
}
