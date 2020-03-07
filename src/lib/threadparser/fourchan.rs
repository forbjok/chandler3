use std::borrow::Cow;
use std::collections::VecDeque;

use super::*;
use crate::html;

use html5ever::local_name;
use kuchiki::*;

pub struct FourchanThread {
    pub root: NodeRef,
}

pub struct FourchanPost {
    pub id: u32,
    pub node: NodeRef,
}

struct GetPosts {
    posts: VecDeque<NodeRef>,
}

impl Iterator for GetPosts {
    type Item = FourchanPost;

    fn next(&mut self) -> Option<FourchanPost> {
        self.posts.pop_front().map(|node| FourchanPost {
            id: get_post_id(node.clone()).expect("Error getting post ID!"),
            node: node.clone(),
        })
    }
}

impl MergeableImageboardThread for FourchanThread {
    type Document = NodeRef;
    type Post = FourchanPost;

    fn from_document(document: Self::Document) -> Self {
        Self { root: document }
    }

    fn into_document(self) -> Self::Document {
        self.root
    }

    fn get_all_posts(&self) -> Result<Box<dyn Iterator<Item = Self::Post>>, ThreadError> {
        let thread_element = html::find_elements(self.root.clone(), local_name!("div"), &["thread"])
            .next()
            .ok_or_else(|| ThreadError::Other(Cow::Borrowed("Error getting thread element!")))?;

        let posts = thread_element.children().collect();

        Ok(Box::new(GetPosts { posts }))
    }

    fn merge_posts_from(&mut self, other: &Self) -> Result<(), ThreadError> {
        let last_main_post = self
            .get_all_posts()?
            .last()
            .ok_or_else(|| ThreadError::Other(Cow::Borrowed("Could not get last post!")))?;

        let main_post_parent = last_main_post
            .node
            .parent()
            .ok_or_else(|| ThreadError::Other(Cow::Borrowed("Could not get main post parent node!")))?;

        let mut other_thread_post_iter = other.get_all_posts()?;

        while let Some(other_post) = other_thread_post_iter.next() {
            if other_post.id <= last_main_post.id {
                continue;
            }

            // Append it to main thread
            main_post_parent.append(other_post.node);
        }

        Ok(())
    }
}

fn get_post_id(node: NodeRef) -> Option<u32> {
    match node.data() {
        NodeData::Element(data) => {
            // Try to locate "id" attribute
            if let Some(id_attr) = data.attributes.borrow().get(local_name!("id")) {
                // Try to parse it as an integer, skipping the "pc" prefix
                if let Ok(id) = id_attr[2..].parse::<u32>() {
                    return Some(id);
                }
            }

            None
        }

        _ => None,
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
