mod find_elements;

use kuchiki::*;

pub use self::find_elements::*;

pub fn parse_string(html_str: &str) -> NodeRef {
    use html5ever::driver::ParseOpts;
    use html5ever::tendril::TendrilSink;
    use html5ever::tree_builder::TreeBuilderOpts;

    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: false,
            ..Default::default()
        },
        ..Default::default()
    };

    kuchiki::parse_html()
        .from_utf8()
        .one(html_str.as_bytes())
}

pub fn  to_string(node: NodeRef) -> String {
    let mut serialized = Vec::new();
    html5ever::serialize(&mut serialized, &node, Default::default()).unwrap();

    String::from_utf8(serialized).unwrap()
}
