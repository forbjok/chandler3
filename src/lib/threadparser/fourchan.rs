use std::borrow::Cow;

use super::*;
use crate::html;

use html5ever::{local_name};
use kuchiki::*;

pub struct FourchanThread {
    pub dom: NodeRef,
}

pub struct FourchanPost {
    pub id: u32,
    pub handle: NodeRef,
}

struct GetAllPosts {
    post_iter: Box<dyn Iterator<Item = NodeRef>>,
}

impl Iterator for GetAllPosts {
    type Item = FourchanPost;

    fn next(&mut self) -> Option<FourchanPost> {
        if let Some(post_handle) = self.post_iter.next() {
            return Some(FourchanPost {
                id: get_post_id(post_handle.clone()).expect("Error getting post ID!"),
                handle: post_handle.clone(),
            });
        }

        None
    }
}

impl MergeableImageboardThread for FourchanThread {
    type Document = NodeRef;
    type Post = FourchanPost;

    fn from_document(document: Self::Document) -> Self {
        Self {
            dom: document,
        }
    }

    fn into_document(self) -> Self::Document {
        self.dom
    }

    fn get_all_posts(&self) -> Result<Box<dyn Iterator<Item = FourchanPost>>, ThreadError> {
        let thread_element = html::find_elements(self.dom.clone(), local_name!("div"), vec!["thread"]).next()
            .ok_or_else(|| ThreadError::Other(Cow::Borrowed("Error getting thread element!")))?;

        let posts: Vec<NodeRef> = thread_element.children().map(|c| c.clone()).collect();

        Ok(Box::new(GetAllPosts {
            post_iter: Box::new(posts.into_iter()),
        }))
    }

    fn merge_posts_from(&mut self, mut other: Self) -> Result<(), ThreadError> {
        let last_main_post = self.get_all_posts()?.last()
            .ok_or_else(|| ThreadError::Other(Cow::Borrowed("Could not get last post!")))?;

        let main_post_parent = last_main_post.handle.parent()
            .ok_or_else(|| ThreadError::Other(Cow::Borrowed("Could not get main post parent node!")))?;

        let mut other_thread_post_iter = other.get_all_posts()?;

        while let Some(other_post) = other_thread_post_iter.next() {
            if other_post.id <= last_main_post.id {
                continue;
            }

            // Detach post from other thread
            other_post.handle.detach();

            // Create new node
            //let new_node = Node::new(other_post.handle.clone().data);

            // Append it to main thread
            main_post_parent.append(other_post.handle);
        }

        Ok(())
    }
}

fn get_post_id(node: NodeRef) -> Option<u32> {
    match node.data() {
        NodeData::Element(data) => {
            // Try to locate "id" attribute
            if let Some(id_attr) = data.attributes.borrow().get("id") {
                // Try to parse it as an integer, skipping the "pc" prefix
                if let Ok(id) = id_attr[2..].parse::<u32>() {
                    return Some(id);
                }
            }

            None
        },

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::html::*;
    use crate::parsers::*;

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
        let dom1 = parse_string(THREAD1);
        let dom2 = parse_string(THREAD2);
        let dom3 = parse_string(THREAD3);
        let merged_dom = parse_string(THREAD_MERGED);

        let expected_html = dom_to_string(merged_dom);

        let mut thread1 = FourchanThread::from_document(dom1);
        let thread2 = FourchanThread::from_document(dom2);
        let thread3 = FourchanThread::from_document(dom3);

        thread1.merge_posts_from(thread2).unwrap();
        thread1.merge_posts_from(thread3).unwrap();

        let dom1 = thread1.into_document();

        let result_html = dom_to_string(dom1);
        assert_eq!(result_html, expected_html);
    }
}
