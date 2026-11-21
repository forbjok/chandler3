use std::sync::LazyLock;

use ego_tree::NodeMut;
use html5ever::{LocalName, Namespace, QualName, local_name};
use scraper::{Html, Node, Selector, node::Element};

#[derive(Debug)]
pub enum LinkTag {
    A,
    Img,
    Link,
}

pub struct LinkMut<'a> {
    element: &'a mut Element,
    tag: LinkTag,
}

static A_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("a").unwrap());
static IMG_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("img").unwrap());
static LINK_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("link").unwrap());

fn for_links_mut(
    html: &mut Html,
    mut action: impl FnMut(LinkMut) -> Result<(), ChandlerError>,
) -> Result<(), ChandlerError> {
    let mut ids: Vec<_> = [&A_SELECTOR, &IMG_SELECTOR, &LINK_SELECTOR]
        .iter()
        .map(|sel| html.select(sel))
        .flatten()
        .map(|e| e.id())
        .collect();

    for id in ids.into_iter() {
        let Some(node) = html.tree.get_mut(id) else { continue };

        let Some(link) = LinkMut::from_node_mut(&mut node) else {
            continue;
        };

        action(link)?;
    }

    Ok(())
}

impl LinkTag {
    pub fn from_name(name: &LocalName) -> Option<Self> {
        match *name {
            local_name!("a") => Some(Self::A),
            local_name!("img") => Some(Self::Img),
            local_name!("link") => Some(Self::Link),
            _ => None,
        }
    }

    pub fn attr_name(&self) -> LocalName {
        match self {
            LinkTag::A => local_name!("href"),
            LinkTag::Img => local_name!("src"),
            LinkTag::Link => local_name!("link"),
        }
    }
}

impl<'a> LinkMut<'a> {
    pub fn from_element(element: &'a mut Element) -> Option<Self> {
        let Some(tag) = LinkTag::from_name(&element.name.local) else {
            return None;
        };

        Some(Self { element, tag })
    }
    pub fn from_node_mut(node: &'a mut NodeMut<'a, Node>) -> Option<Self> {
        let Node::Element(el) = node.value() else { return None };

        Self::from_element(el)
    }

    pub fn link(&self) -> Option<String> {
        let attr_name = self.tag.attr_name();

        self.element.attr(&attr_name).map(|v| v.to_owned())
    }

    pub fn file_link(&self) -> Option<String> {
        self.link().filter(|link| {
            if link.is_empty() {
                return false;
            }

            if link.starts_with('#') {
                return false;
            }

            if link.ends_with('/') {
                return false;
            }

            if link.starts_with("javascript:") {
                return false;
            }

            true
        })
    }

    pub fn replace(&mut self, with: &str) {
        let attr_name = self.tag.attr_name();
        let Some(attr) = self.element.attrs.iter_mut().find(|(n, _)| n.local == attr_name) else {
            return;
        };

        let original_value = std::mem::replace(&mut attr.1, with.into());

        let original_qualname = QualName::new(
            None,
            Namespace::default(),
            format!("data-original-{}", attr_name).into(),
        );
        self.element.attrs.push((original_qualname, original_value));
    }
}
