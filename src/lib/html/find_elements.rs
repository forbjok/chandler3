use std::collections::VecDeque;

use html5ever::{local_name, LocalName};
use kuchiki::*;

use crate::util::IsSubset;

pub struct FindElements<P> {
    queue: VecDeque<NodeRef>,
    find_name: LocalName,
    predicate: P,
}

impl<P> Iterator for FindElements<P>
where
    P: Fn(&ElementData) -> bool,
{
    type Item = NodeRef;

    fn next(&mut self) -> Option<NodeRef> {
        // Grab next node from the queue
        while let Some(node) = self.queue.pop_front() {
            let mut is_match = false;

            match node.data() {
                NodeData::Element(data) => {
                    let name = &data.name;

                    if name.local == self.find_name {
                        let predicate = &self.predicate;

                        is_match = predicate(data);
                    }
                }

                _ => {}
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

pub fn find_elements<P>(node: NodeRef, find_name: impl Into<LocalName>, predicate: P) -> FindElements<P>
where
    P: Fn(&ElementData) -> bool,
{
    FindElements {
        queue: Some(node).into_iter().collect(),
        find_name: find_name.into(),
        predicate,
    }
}

pub fn find_elements_with_classes<'a>(
    node: NodeRef,
    find_name: impl Into<LocalName>,
    find_classes: &[&'a str],
) -> impl Iterator<Item = NodeRef> + 'a {
    let find_classes: Vec<&'a str> = find_classes.iter().map(|s| *s).collect();

    find_elements(node, find_name, move |data: &ElementData| {
        if let Some(class_attr) = data.attributes.borrow().get(local_name!("class")) {
            // Split classes into a vector of strings
            let classes = class_attr.split(' ').collect::<Vec<&str>>();

            // Does the element have the specified classes?
            find_classes.is_subset(&classes)
        } else {
            false
        }
    })
}
