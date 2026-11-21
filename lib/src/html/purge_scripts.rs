use std::sync::LazyLock;

use scraper::{Html, Selector};

static SCRIPT_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("script").unwrap());

/// Remove all script tags in this Kuchiki node and all children
pub fn purge_scripts(html: &mut Html) {
    let ids: Vec<_> = html.select(&SCRIPT_SELECTOR).map(|e| e.id()).collect();

    for id in ids {
        let Some(mut node) = html.tree.get_mut(id) else {
            continue;
        };

        node.detach();
    }
}
