use scraper::{Html, Selector};

/// Remove all script tags in this Kuchiki node and all children
pub fn purge_scripts(html: &mut Html) {
    let selector = Selector::parse("script").unwrap();

    let ids: Vec<_> = html.select(&selector).map(|e| e.id()).collect();

    for id in ids {
        let Some(mut node) = html.tree.get_mut(id) else {
            continue;
        };

        node.detach();
    }
}
