use html5ever::{local_name, LocalName};
use kuchiki::*;

use crate::util::IsSubset;

pub struct FindElements<'a> {
    queue: Vec<NodeRef>,
    find_name: LocalName,
    find_classes: Vec<&'a str>,
}

impl<'a> Iterator for FindElements<'a> {
    type Item = NodeRef;

    fn next(&mut self) -> Option<NodeRef> {
        loop {
            let mut is_match = false;

            // If node queue is empty, break out of the loop.
            if self.queue.is_empty() {
                break;
            }

            // Grab next node from the queue
            let node = self.queue.remove(0);

            match node.data() {
                NodeData::Element(data) => {
                    let name = &data.name;

                    if name.local == self.find_name {
                        if let Some(class_attr) = data.attributes.borrow().get(local_name!("class")) {
                            // Split classes into a vector of strings
                            let classes = class_attr.split(' ').collect::<Vec<&str>>();

                            // Does the element have the specified classes?
                            if self.find_classes.is_subset(&classes) {
                                is_match = true;
                            }
                        }
                    }
                },

                _ => { },
            };

            // Add child nodes to queue
            self.queue.extend(node.children());

            // If the node matched, return it.
            if is_match {
                return Some(node);
            }
        }

        None
    }
}

pub fn find_elements<'a>(node: NodeRef, find_name: impl Into<LocalName>, find_classes: &[&'a str]) -> FindElements<'a> {
    FindElements {
        queue: vec![node],
        find_name: find_name.into(),
        find_classes: find_classes.iter().map(|s| *s).collect(),
    }
}
