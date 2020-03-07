use std::borrow::Cow;
use std::fmt;

#[derive(Debug)]
pub enum ThreadError {
    Other(Cow<'static, str>),
}

impl fmt::Display for ThreadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Other(err) => write!(f, "{}", err),
        }
    }
}

pub trait MergeableImageboardThread: Sized {
    type Document;
    type Post;

    fn from_document(document: Self::Document) -> Self;
    fn into_document(self) -> Self::Document;

    fn get_all_posts(&self) -> Result<Box<dyn Iterator<Item = Self::Post>>, ThreadError>;

    fn merge_posts_from(&mut self, other: &Self) -> Result<(), ThreadError>;
}

pub mod fourchan;
