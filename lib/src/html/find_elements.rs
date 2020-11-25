use std::collections::VecDeque;

use html5ever::{local_name, LocalName};
use kuchiki::*;

use crate::util::IsSubset;

pub struct FindElements<P> {
    queue: VecDeque<NodeRef>,
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

            if let NodeData::Element(data) = node.data() {
                let predicate = &self.predicate;

                is_match = predicate(data);
            }

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

pub fn find_elements<P>(node: NodeRef, predicate: P) -> FindElements<P>
where
    P: Fn(&ElementData) -> bool,
{
    FindElements {
        queue: Some(node).into_iter().collect(),
        predicate,
    }
}

pub fn find_elements_with_classes<'a>(
    node: NodeRef,
    find_name: impl Into<LocalName>,
    find_classes: &[&'a str],
) -> impl Iterator<Item = NodeRef> + 'a {
    let find_name = find_name.into();
    let find_classes: Vec<&'a str> = find_classes.iter().copied().collect();

    find_elements(node, move |data: &ElementData| {
        if data.name.local == find_name {
            if let Some(class_attr) = data.attributes.borrow().get(local_name!("class")) {
                // Split classes into a vector of strings
                let classes = class_attr.split(' ').collect::<Vec<&str>>();

                // Does the element have the specified classes?
                if find_classes.is_subset(&classes) {
                    return true;
                }
            }
        }

        false
    })
}
