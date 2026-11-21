use std::collections::VecDeque;

use ego_tree::{NodeMut, NodeRef};
use scraper::Node;

pub fn copy_subtree(src: NodeRef<'_, Node>, dst: &mut NodeMut<'_, Node>, insert_root_node: F)
where
    F: FnMut(Node, NodeMut<'_, Node>) -> NodeMut<'_, Node>,
{
    let new_root = insert_root_node(src.value().clone(), dst);
    let dst_id = new_root.id();
    let dst_tree = new_root.tree();

    let mut nodes: VecDeque<_> = src.children().map(|c| (c, dst_id)).collect();

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
