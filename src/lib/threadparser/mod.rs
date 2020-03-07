use std::borrow::Cow;
use std::fmt;
use std::path::Path;

use crate::error::*;
use crate::html;

pub trait MergeableImageboardThread: Sized {
    type Document;
    type Post;

    fn from_document(document: Self::Document) -> Self;
    fn into_document(self) -> Self::Document;

    fn from_file(file_path: &Path) -> Result<Self, ChandlerError>;
    fn write_file(&self, file_path: &Path) -> Result<(), ChandlerError>;

    fn get_all_posts(&self) -> Result<Box<dyn Iterator<Item = Self::Post>>, ChandlerError>;

    fn merge_posts_from(&mut self, other: &Self) -> Result<Vec<Self::Post>, ChandlerError>;

    fn for_links(&self, action: impl FnMut(&mut html::Link)) -> Result<(), ChandlerError>;
    fn for_post_links(&self, post: &Self::Post, action: impl FnMut(&mut html::Link)) -> Result<(), ChandlerError>;

    /// Purge all script tags
    fn purge_scripts(&self) -> Result<(), ChandlerError>;
}

pub mod fourchan;
