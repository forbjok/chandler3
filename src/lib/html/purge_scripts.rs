use html5ever::local_name;
use kuchiki::*;

use crate::html;

/// Remove all script tags in this Kuchiki node and all children
pub fn purge_scripts(node: NodeRef) {
    for node in html::find_elements(node, |ed| ed.name.local == local_name!("script")) {
        node.detach();
    }
}
