use std::borrow::Cow;
use std::collections::VecDeque;

use super::*;
use crate::error::ChandlerError;
use crate::html;
use crate::util;

use html5ever::local_name;
use kuchiki::*;

pub struct FourchanThread {
    pub root: NodeRef,
}

#[derive(Clone, Debug)]
pub struct FourchanPost {
    pub id: u32,
    pub node: NodeRef,
}

impl FourchanPost {
    pub fn from_node(node: NodeRef) -> Option<Self> {
        // Try to get post ID from node
        let id = (|| {
            if let NodeData::Element(data) = node.data() {
                // Try to locate "id" attribute
                if let Some(id_attr) = data.attributes.borrow().get(local_name!("id")) {
                    // Try to parse it as an integer, skipping the "pc" prefix
                    if let Ok(id) = id_attr[2..].parse::<u32>() {
                        return Some(id);
                    }
                }
            }

            None
        })();

        // Convert ID and node into post if found
        id.map(|id| Self { id, node })
    }
}

struct GetPosts {
    posts: VecDeque<NodeRef>,
}

impl Iterator for GetPosts {
    type Item = FourchanPost;

    fn next(&mut self) -> Option<FourchanPost> {
        while let Some(node) = self.posts.pop_front() {
            if let Some(post) = FourchanPost::from_node(node) {
                return Some(post);
            }
        }

        None
    }
}

impl HtmlDocument for FourchanThread {
    type Document = NodeRef;

    fn from_document(document: Self::Document) -> Self {
        Self { root: document }
    }

    fn into_document(self) -> Self::Document {
        self.root
    }

    fn from_file(file_path: &Path) -> Result<Self, ChandlerError> {
        let node = html::parse_file(file_path)?;

        Ok(Self::from_document(node))
    }

    fn write_file(&self, file_path: &Path) -> Result<(), ChandlerError> {
        let mut file = util::create_file(file_path).map_err(ChandlerError::CreateFile)?;

        html5ever::serialize(&mut file, &self.root, Default::default())
            .map_err(|err| ChandlerError::Other(Cow::Owned(format!("Serialization error: {}", err))))?;

        Ok(())
    }

    fn for_links(&self, mut action: impl FnMut(html::Link) -> Result<(), ChandlerError>) -> Result<(), ChandlerError> {
        let links = html::find_links(self.root.clone());

        for link in links.into_iter() {
            action(link)?;
        }

        Ok(())
    }

    fn purge_scripts(&self) -> Result<(), ChandlerError> {
        html::purge_scripts(self.root.clone());

        Ok(())
    }
}

impl MergeableImageboardThread for FourchanThread {
    type Post = FourchanPost;

    fn get_all_posts(&self) -> Result<Box<dyn Iterator<Item = Self::Post>>, ChandlerError> {
        let thread_element = html::find_elements_with_classes(self.root.clone(), local_name!("div"), &["thread"])
            .next()
            .ok_or_else(|| ChandlerError::Other("Error getting thread element!".into()))?;

        let posts = thread_element.children().collect();

        Ok(Box::new(GetPosts { posts }))
    }

    fn merge_posts_from(&mut self, other: &Self) -> Result<Vec<Self::Post>, ChandlerError> {
        let last_main_post = self
            .get_all_posts()?
            .last()
            .ok_or_else(|| ChandlerError::Other("Could not get last post!".into()))?;

        let main_post_parent = last_main_post
            .node
            .parent()
            .ok_or_else(|| ChandlerError::Other("Could not get main post parent node!".into()))?;

        let mut new_posts: Vec<FourchanPost> = Vec::new();

        for other_post in other.get_all_posts()? {
            if other_post.id <= last_main_post.id {
                continue;
            }

            // Append it to main thread
            main_post_parent.append(other_post.node.clone());

            new_posts.push(other_post);
        }

        Ok(new_posts)
    }

    fn for_post_links(
        &self,
        post: &Self::Post,
        mut action: impl FnMut(html::Link) -> Result<(), ChandlerError>,
    ) -> Result<(), ChandlerError> {
        let links = html::find_links(post.node.clone());

        for link in links.into_iter() {
            action(link)?;
        }

        Ok(())
    }

    fn is_archived(&self) -> Result<bool, ChandlerError> {
        Ok(html::find_elements_with_classes(self.root.clone(), local_name!("img"), &["archivedIcon"]).any(|_| true))
    }
}

#[cfg(test)]
mod tests {
    use crate::html;

    use super::*;

    // Original thread with OP only
    const THREAD1: &'static str = r#"<div class="thread" id="t1"><div class="postContainer" id="pc1"></div></div>"#;

    // Thread with 2 posts
    const THREAD2: &'static str = r#"<div class="thread" id="t1"><div class="postContainer" id="pc1"></div><div class="postContainer" id="pc2"></div></div>"#;

    // Thread with post 2 deleted and a new post 3 added
    const THREAD3: &'static str = r#"<div class="thread" id="t1"><div class="postContainer" id="pc1"></div><div class="postContainer" id="pc3"></div></div>"#;

    // Merged thread with all 3 posts
    const THREAD_MERGED: &'static str = r#"<div class="thread" id="t1"><div class="postContainer" id="pc1"></div><div class="postContainer" id="pc2"></div><div class="postContainer" id="pc3"></div></div>"#;

    #[test]
    fn can_merge_threads() {
        let node1 = html::parse_string(THREAD1);
        let node2 = html::parse_string(THREAD2);
        let node3 = html::parse_string(THREAD3);
        let merged_node = html::parse_string(THREAD_MERGED);

        let expected_html = html::to_string(merged_node);

        let mut thread1 = FourchanThread::from_document(node1);
        let thread2 = FourchanThread::from_document(node2);
        let thread3 = FourchanThread::from_document(node3);

        thread1.merge_posts_from(&thread2).unwrap();
        thread1.merge_posts_from(&thread3).unwrap();

        let node1 = thread1.into_document();

        let result_html = html::to_string(node1);

        assert_eq!(result_html, expected_html);
    }
}
