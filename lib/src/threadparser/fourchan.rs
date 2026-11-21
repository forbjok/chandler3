use std::borrow::Cow;
use std::collections::VecDeque;
use std::io::BufWriter;

use super::*;
use crate::error::ChandlerError;
use crate::html;
use crate::util;

use html5ever::local_name;
use scraper::ElementRef;
use scraper::Html;

pub struct FourchanThread {
    pub html: Html,
}

#[derive(Clone, Debug)]
pub struct FourchanReply<'a> {
    pub id: u32,
    pub element: ElementRef<'a>,
}

impl<'a> FourchanReply<'a> {
    pub fn from_node(element: ElementRef<'a>) -> Option<Self> {
        let id = element
            .attr("id")
            .and_then(|v| v.strip_prefix("pc").and_then(|v| v.parse::<u32>().ok()))?;

        Some(Self { id, element })
    }
}

struct GetReplies<'a> {
    replies: VecDeque<ElementRef<'a>>,
}

impl<'a> Iterator for GetReplies<'a> {
    type Item = FourchanReply<'a>;

    fn next(&mut self) -> Option<FourchanReply<'a>> {
        while let Some(node) = self.replies.pop_front() {
            if let Some(reply) = FourchanReply::from_node(node) {
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

    fn for_links(&self, mut action: impl FnMut(html::Link) -> Result<(), ChandlerError>) -> Result<(), ChandlerError> {
        let links = html::find_links(self.html.clone());

        for link in links.into_iter() {
            action(link)?;
        }

        Ok(())
    }

    fn purge_scripts(&mut self) -> Result<(), ChandlerError> {
        html::purge_scripts(&mut self.html);

        Ok(())
    }
}

impl MergeableImageboardThread for FourchanThread {
    type Reply = FourchanReply;

    fn get_all_replies(&self) -> Result<Box<dyn Iterator<Item = Self::Reply>>, ChandlerError> {
        let thread_element = html::find_elements_with_classes(self.root.clone(), local_name!("div"), &["thread"])
            .next()
            .ok_or_else(|| ChandlerError::Other("No thread element found!".into()))?;

        let replies = thread_element.children().skip(1).collect();

        Ok(Box::new(GetReplies { replies }))
    }

    fn merge_replies_from(&mut self, new: Self) -> Result<Vec<Self::Reply>, ChandlerError> {
        // Create temporary insert marker node.
        let insert_marker_node = NodeRef::new_comment("INSERT");

        let last_reply_id = match self.get_all_replies()?.last() {
            Some(last_original_reply) => {
                last_original_reply.node.insert_after(insert_marker_node.clone());

                last_original_reply.id
            }
            _ => {
                // Get original thread element.
                let original_thread_element =
                    html::find_elements_with_classes(self.root.clone(), local_name!("div"), &["thread"])
                        .next()
                        .ok_or_else(|| ChandlerError::Other("No thread element found in original thread!".into()))?;

                // Append the insert marker node to the original thread element.
                original_thread_element.append(insert_marker_node.clone());

                0
            }
        };

        let mut new_replies: Vec<Self::Reply> = Vec::new();

        for new_reply in new.get_all_replies()? {
            if new_reply.id <= last_reply_id {
                continue;
            }

            // Append it to original thread.
            insert_marker_node.insert_before(new_reply.node.clone());

            new_replies.push(new_reply);
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
        Ok(html::find_elements_with_classes(self.root.clone(), local_name!("img"), &["archivedIcon"]).any(|_| true))
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
