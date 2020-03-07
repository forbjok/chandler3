use kuchiki::*;

use crate::html;

/// Remove all script tags in this Kuchiki node and all children
pub fn purge_scripts(node: NodeRef) {
    for node in html::find_elements(node, "script", |_| true) {
        node.detach();
    }
}
