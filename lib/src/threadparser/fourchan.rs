use std::collections::VecDeque;
use std::sync::LazyLock;

use super::*;
use crate::error::ChandlerError;
use crate::html;

use ego_tree::NodeId;
use ego_tree::NodeRef;
use scraper::ElementRef;
use scraper::Html;
use scraper::Node;
use scraper::Selector;
use scraper::node::Comment;

pub struct FourchanThread {
    pub html: Html,
}

#[derive(Clone, Debug)]
pub struct FourchanReply {
    pub id: u32,
    pub node_id: NodeId,
}

impl FourchanReply {
    pub fn from_node(node: NodeRef<'_, Node>) -> Option<Self> {
        let Node::Element(element) = node.value() else {
            return None;
        };

        let id = element
            .attr("id")
            .and_then(|v| v.strip_prefix("pc"))
            .and_then(|v| v.parse::<u32>().ok())?;

        Some(Self { id, node_id: node.id() })
    }
}

struct GetReplies<'a> {
    nodes: VecDeque<NodeRef<'a, Node>>,
}

static ARCHIVED_ICON_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("img.archivedIcon").unwrap());
static THREAD_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("div.thread").unwrap());

impl FourchanThread {
    fn get_thread_element(&self) -> Option<ElementRef> {
        self.html.select(&THREAD_SELECTOR).next()
    }
}

impl<'a> Iterator for GetReplies<'a> {
    type Item = FourchanReply;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.nodes.pop_front() {
            if let Some(reply) = Self::Item::from_node(node) {
                return Some(reply);
            }
        }

        None
    }
}

impl HtmlDocument for FourchanThread {
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
        action: impl FnMut(html::LinkMut) -> Result<(), ChandlerError>,
    ) -> Result<(), ChandlerError> {
        html::for_links_mut(&mut self.html, None, action)
    }

    fn purge_scripts(&mut self) -> Result<(), ChandlerError> {
        html::purge_scripts(&mut self.html);

        Ok(())
    }
}

impl MergeableImageboardThread for FourchanThread {
    type Reply = FourchanReply;

    fn get_all_replies<'a>(&'a self) -> Result<Box<dyn Iterator<Item = Self::Reply> + 'a>, ChandlerError> {
        let thread_element = self
            .get_thread_element()
            .ok_or_else(|| ChandlerError::Other("No thread element found!".into()))?;

        let nodes: VecDeque<_> = thread_element.children().skip(1).collect();

        Ok(Box::new(GetReplies { nodes }))
    }

    fn merge_replies_from(&mut self, new: Self) -> Result<Vec<Self::Reply>, ChandlerError> {
        // Create temporary insert marker node.
        let mut insert_marker_node;

        let last_reply_id = match self.get_all_replies()?.last() {
            Some(last_original_reply) => {
                let mut last_original_reply_node = self.html.tree.get_mut(last_original_reply.node_id).unwrap();
                insert_marker_node = last_original_reply_node.insert_after(Node::Comment(Comment {
                    comment: "INSERT".into(),
                }));

                last_original_reply.id
            }
            _ => {
                let thread_element = self
                    .get_thread_element()
                    .ok_or_else(|| ChandlerError::Other("Could not get thread element".into()))?;

                let thread_element = self.html.tree.get_mut(thread_element.id()).unwrap();
                insert_marker_node = thread_element.append(Node::Comment(Comment {
                    comment: "INSERT".into(),
                }));

                0
            }
        };

        let mut new_replies: Vec<Self::Reply> = Vec::new();

        let new_replies: Vec<_> = new.get_all_replies()?.filter(|r| r.id <= last_reply_id).collect();

        for new_reply in new_replies {
            let reply_node = new
                .html
                .tree
                .get(new_reply.node_id)
                .ok_or_else(|| ChandlerError::Other("Could not get reply node from source".into()))?;

            html::copy_subtree(reply_node, &mut insert_marker_node, |src, dst| dst.insert_before(src));
        }

        // Remove temporary insert marker node.
        insert_marker_node.detach();

        Ok(new_replies)
    }

    fn for_reply_links_mut(
        &mut self,
        reply: &mut Self::Reply,
        mut action: impl FnMut(html::LinkMut) -> Result<(), ChandlerError>,
    ) -> Result<(), ChandlerError> {
        html::for_links_mut(&mut self.html, Some(reply.node_id), action)
    }

    fn is_archived(&self) -> Result<bool, ChandlerError> {
        Ok(self.html.select(&ARCHIVED_ICON_SELECTOR).next().is_some())
    }
}

#[cfg(test)]
mod tests {
    use crate::html;

    use super::*;

    // Original thread with OP only
    const THREAD1: &'static str = r#"<div class="thread" id="t1"><div class="opContainer" id="pc1"></div></div>"#;

    // Thread with 2 posts
    const THREAD2: &'static str = r#"<div class="thread" id="t1"><div class="opContainer" id="pc1"></div><div class="replyContainer" id="pc2"></div></div>"#;

    // Thread with post 2 deleted and a new post 3 added
    const THREAD3: &'static str = r#"<div class="thread" id="t1"><div class="opContainer" id="pc1"></div><div class="replyContainer" id="pc3"></div></div>"#;

    // Merged thread with all 3 posts
    const THREAD_MERGED: &'static str = r#"<div class="thread" id="t1"><div class="opContainer" id="pc1"></div><div class="replyContainer" id="pc2"></div><div class="replyContainer" id="pc3"></div></div>"#;

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

        thread1.merge_replies_from(thread2).unwrap();
        thread1.merge_replies_from(thread3).unwrap();

        let node1 = thread1.into_document();

        let result_html = html::to_string(node1);

        assert_eq!(result_html, expected_html);
    }
}
