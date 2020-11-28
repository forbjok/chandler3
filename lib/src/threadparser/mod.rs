use std::path::Path;

use crate::error::*;
use crate::html;

pub mod basic;
pub mod fourchan;
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
    type Post;

    fn get_all_posts(&self) -> Result<Box<dyn Iterator<Item = Self::Post>>, ChandlerError>;

    fn merge_posts_from(&mut self, other: &Self) -> Result<Vec<Self::Post>, ChandlerError>;

    fn for_post_links(
        &self,
        post: &Self::Post,
        action: impl FnMut(html::Link) -> Result<(), ChandlerError>,
    ) -> Result<(), ChandlerError>;

    fn is_archived(&self) -> Result<bool, ChandlerError>;
}
