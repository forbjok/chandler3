use html5ever::local_name;
use kuchikiki::*;

use crate::util::IsSubset;

pub fn has_classes(data: &ElementData, class_names: &[&str]) -> bool {
    if let Some(class_attr) = data.attributes.borrow().get(local_name!("class")) {
        // Split classes into a vector of strings
        let classes: Vec<&str> = class_attr.split(' ').collect();

        // Does the element have the specified classes?
        class_names.is_subset(&classes)
    } else {
        false
    }
}
