mod file;
mod find_elements;
mod find_links;
mod purge_scripts;

use kuchiki::*;

pub use self::file::*;
pub use self::find_elements::*;
pub use self::find_links::*;
pub use self::purge_scripts::*;

/// Parse string into Kuchiki node
pub fn parse_string(html_str: &str) -> NodeRef {
    use html5ever::tendril::TendrilSink;

    kuchiki::parse_html().from_utf8().one(html_str.as_bytes())
}

/// Serialize Kuchiki node to string
pub fn to_string(node: NodeRef) -> String {
    let mut serialized = Vec::new();
    html5ever::serialize(&mut serialized, &node, Default::default()).unwrap();

    String::from_utf8(serialized).unwrap()
}

/// Round-trip the HTML string through Kuchiki
pub fn normalize(html_str: &str) -> String {
    to_string(parse_string(html_str))
}