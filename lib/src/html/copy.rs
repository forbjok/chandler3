use std::collections::VecDeque;

use ego_tree::{NodeMut, NodeRef};
use scraper::Node;

pub fn copy_subtree(src: NodeRef<'_, Node>, mut dst: NodeMut<'_, Node>) {
    let dst_id = dst.id();
    let dst_tree = dst.tree();

    let mut nodes = VecDeque::from([(src, dst_id)]);

    while let Some((src, dst_id)) = nodes.pop_front() {
        let Some(mut dst) = dst_tree.get_mut(dst_id) else {
            continue;
        };

        let new_node = dst.append(src.value().clone());

        for child in src.children() {
            nodes.push_back((child, new_node.id()));
        }
    }
}
