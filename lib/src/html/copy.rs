use std::collections::VecDeque;

use ego_tree::{NodeMut, NodeRef};
use scraper::Node;

pub fn copy_subtree<'a, F>(src: NodeRef<'a, Node>, dst: &mut NodeMut<'a, Node>, insert_root_node: F)
where
    F: Fn(Node, &mut NodeMut<'a, Node>) -> NodeMut<'a, Node>,
{
    let mut new_root = insert_root_node(src.value().clone(), dst);
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
