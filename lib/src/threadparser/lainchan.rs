use std::borrow::Cow;
use std::collections::VecDeque;
use std::iter;

use html5ever::{local_name, namespace_url, ns, QualName};
use kuchiki::*;
use lazy_static::lazy_static;
use regex::Regex;

use crate::error::ChandlerError;
use crate::html;
use crate::util;

use super::*;

lazy_static! {
    static ref REGEX_GET_REPLY_ID: Regex = Regex::new(r#"^pc(\d+)"#).unwrap();
}

pub struct LainchanThread {
    pub root: NodeRef,
}

#[derive(Clone, Debug)]
pub struct LainchanReply {
    pub id: u32,
    pub node: NodeRef,
}

impl LainchanReply {
    pub fn from_node(node: NodeRef) -> Option<Self> {
        // Try to get reply ID from node.
        let id = (|| {
            if let NodeData::Element(data) = node.data() {
                // Try to locate "id" attribute.
                if let Some(id_attr) = data.attributes.borrow().get(local_name!("id")) {
                    // Try to get post ID from attribute.
                    if let Some(caps) = REGEX_GET_REPLY_ID.captures(id_attr) {
                        return Some(caps[1].parse::<u32>().unwrap());
                    }
                }
            }

            None
        })();

        // Convert ID and node into reply if found.
        id.map(|id| Self { id, node })
    }
}

struct GetReplies {
    replies: VecDeque<NodeRef>,
}

impl Iterator for GetReplies {
    type Item = LainchanReply;

    fn next(&mut self) -> Option<LainchanReply> {
        while let Some(node) = self.replies.pop_front() {
            if let Some(reply) = LainchanReply::from_node(node) {
                return Some(reply);
            }
        }

        None
    }
}

impl HtmlDocument for LainchanThread {
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

impl MergeableImageboardThread for LainchanThread {
    type Reply = LainchanReply;

    fn get_all_replies(&self) -> Result<Box<dyn Iterator<Item = Self::Reply>>, ChandlerError> {
        let replies: VecDeque<NodeRef> =
            html::find_elements_with_classes(self.root.clone(), local_name!("div"), &["postcontainer"]).collect();

        Ok(Box::new(GetReplies { replies }))
    }

    fn merge_replies_from(&mut self, new: Self) -> Result<Vec<Self::Reply>, ChandlerError> {
        // Create temporary insert marker node.
        let insert_marker_node = NodeRef::new_comment("INSERT");

        let last_reply_id = if let Some(last_original_reply) = self.get_all_replies()?.last() {
            last_original_reply.node.insert_after(insert_marker_node.clone());

            last_original_reply.id
        } else {
            // Get original OP element.
            let original_op_element = html::find_elements_with_classes(self.root.clone(), local_name!("div"), &["op"])
                .next()
                .ok_or_else(|| ChandlerError::Other("No OP element found in original thread!".into()))?;

            // Insert the insert marker node after the original OP element.
            original_op_element.insert_after(insert_marker_node.clone());

            0
        };

        let mut first_reply = last_reply_id == 0;
        let mut new_replies: Vec<Self::Reply> = Vec::new();

        for new_reply in new.get_all_replies()? {
            if new_reply.id <= last_reply_id {
                continue;
            }

            // Unless it's the first reply, insert a <br> before the reply.
            if !first_reply {
                insert_marker_node.insert_before(NodeRef::new_element(
                    QualName::new(None, ns!(html), local_name!("br")),
                    iter::empty(),
                ));
            }

            // Append the reply to the original thread.
            insert_marker_node.insert_before(new_reply.node.clone());

            new_replies.push(new_reply);

            first_reply = false;
        }

        // Remove temporary insert marker node.
        insert_marker_node.detach();

        Ok(new_replies)
    }

    fn for_reply_links(
        &self,
        reply: &Self::Reply,
        mut action: impl FnMut(html::Link) -> Result<(), ChandlerError>,
    ) -> Result<(), ChandlerError> {
        let links = html::find_links(reply.node.clone());

        for link in links.into_iter() {
            action(link)?;
        }

        Ok(())
    }

    fn is_archived(&self) -> Result<bool, ChandlerError> {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use crate::html;

    use super::*;

    // Original thread with OP only
    const THREAD1: &'static str = r#"<div class="thread" id="thread_1"><div class="post op" id="op_1"></div></div>"#;

    // Thread with 2 posts
    const THREAD2: &'static str = r#"<div class="thread" id="thread_1"><div class="post op" id="op_1"></div><div class="postcontainer" id="pc2"></div></div>"#;

    // Thread with post 2 deleted and a new post 3 added
    const THREAD3: &'static str = r#"<div class="thread" id="thread_1"><div class="post op" id="op_1"></div><div class="postcontainer" id="pc3"></div></div>"#;

    // Merged thread with all 3 posts
    const THREAD_MERGED: &'static str = r#"<div class="thread" id="thread_1"><div class="post op" id="op_1"></div><div class="postcontainer" id="pc2"></div><br><div class="postcontainer" id="pc3"></div></div>"#;

    #[test]
    fn can_merge_threads() {
        let node1 = html::parse_string(THREAD1);
        let node2 = html::parse_string(THREAD2);
        let node3 = html::parse_string(THREAD3);
        let merged_node = html::parse_string(THREAD_MERGED);

        let expected_html = html::to_string(merged_node);

        let mut thread1 = LainchanThread::from_document(node1);
        let thread2 = LainchanThread::from_document(node2);
        let thread3 = LainchanThread::from_document(node3);

        thread1.merge_replies_from(thread2).unwrap();
        thread1.merge_replies_from(thread3).unwrap();

        let node1 = thread1.into_document();

        let result_html = html::to_string(node1);

        assert_eq!(result_html, expected_html);
    }
}
