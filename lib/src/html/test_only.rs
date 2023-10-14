// This module is only used in tests.

use kuchikiki::*;

/// Parse string into Kuchiki node.
pub fn parse_string(html_str: &str) -> NodeRef {
    use html5ever::tendril::TendrilSink;

    kuchikiki::parse_html().from_utf8().one(html_str.as_bytes())
}

/// Serialize Kuchiki node to string.
pub fn to_string(node: NodeRef) -> String {
    let mut serialized = Vec::new();
    html5ever::serialize(&mut serialized, &node, Default::default()).unwrap();

    String::from_utf8(serialized).unwrap()
}

/// Round-trip the HTML string through Kuchiki.
pub fn normalize(html_str: &str) -> String {
    to_string(parse_string(html_str))
}
