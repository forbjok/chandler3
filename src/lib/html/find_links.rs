use html5ever::local_name;
use kuchiki::*;

use super::*;

#[derive(Debug)]
pub enum LinkTag {
    A,
    Img,
    Link,
}

#[derive(Debug)]
pub struct Link {
    node: NodeRef,
    tag: LinkTag,
}

impl Link {
    pub fn from_node(node: NodeRef) -> Option<Self> {
        match node.clone().data() {
            NodeData::Element(data) => {
                match data.name.local {
                    // <a> element
                    local_name!("a") => {
                        return Some(Link { node, tag: LinkTag::A });
                    }
                    // <img> element
                    local_name!("img") => {
                        return Some(Link {
                            node,
                            tag: LinkTag::Img,
                        });
                    }
                    // <link> element
                    local_name!("link") => {
                        return Some(Link {
                            node,
                            tag: LinkTag::Link,
                        });
                    }

                    _ => {}
                }
            }

            _ => {}
        }

        None
    }

    pub fn link(&self) -> Option<String> {
        if let NodeData::Element(data) = self.node.data() {
            let attrs = data.attributes.borrow();

            match self.tag {
                LinkTag::A => {
                    if let Some(href_attr) = attrs.get(local_name!("href")) {
                        return Some(href_attr.to_owned());
                    }
                }
                LinkTag::Img => {
                    if let Some(src_attr) = attrs.get(local_name!("src")) {
                        return Some(src_attr.to_owned());
                    }
                }
                LinkTag::Link => {
                    if let Some(href_attr) = attrs.get(local_name!("href")) {
                        return Some(href_attr.to_owned());
                    }
                }
            }
        }

        None
    }

    pub fn file_link(&self) -> Option<String> {
        self.link().filter(|link| {
            if link.is_empty() {
                return false;
            }

            if link.starts_with("#") {
                return false;
            }

            if link.ends_with("/") {
                return false;
            }

            if link.starts_with("javascript:") {
                return false;
            }

            true
        })
    }

    pub fn replace(&mut self, with: &str) {
        if let NodeData::Element(data) = self.node.data() {
            let mut attrs = data.attributes.borrow_mut();

            match self.tag {
                LinkTag::A => {
                    if let Some(href_attr) = attrs.get_mut(local_name!("href")) {
                        href_attr.clear();
                        href_attr.push_str(with);
                    }
                }
                LinkTag::Img => {
                    if let Some(src_attr) = attrs.get_mut(local_name!("src")) {
                        src_attr.clear();
                        src_attr.push_str(with);
                    }
                }
                LinkTag::Link => {
                    if let Some(href_attr) = attrs.get_mut(local_name!("href")) {
                        href_attr.clear();
                        href_attr.push_str(with);
                    }
                }
            };

            attrs.insert("data-original-href", "ORIGINAL".to_owned());
        }
    }
}

pub fn find_links(node: NodeRef) -> Vec<Link> {
    let links = find_elements(node, |_| true).filter_map(Link::from_node).collect();

    links
}

#[cfg(test)]
mod tests {
    use super::*;

    const HTML: &'static str = r###"
    <div>
        <a href="a"></a>
        <img src="images/file.png">
        <link href="css/style.css" rel="stylesheet">
    </div>
    "###;

    const HTML_FILTER_FILE_LINKS: &'static str = r###"
    <div>
        <!-- File links -->
        <a href="a"></a>
        <img src="images/file.png">
        <link href="css/style.css" rel="stylesheet">

        <!-- Not file links -->
        <a href=""></a>
        <a href="#"></a>
        <a href="path/to/dir/"></a>
        <a href="javascript:doStuff()"></a>
    </div>
    "###;

    #[test]
    fn can_find_links() {
        let node = parse_string(HTML);
        let expected_links = vec!["a".to_owned(), "images/file.png".to_owned(), "css/style.css".to_owned()];

        let links = find_links(node);
        let links: Vec<String> = links.into_iter().filter_map(|link| link.link()).collect();

        assert_eq!(links, expected_links);
    }

    #[test]
    fn can_find_file_links() {
        let node = parse_string(HTML_FILTER_FILE_LINKS);
        let expected_links = vec!["a".to_owned(), "images/file.png".to_owned(), "css/style.css".to_owned()];

        let links = find_links(node);
        let links: Vec<String> = links.into_iter().filter_map(|link| link.file_link()).collect();

        assert_eq!(links, expected_links);
    }
}
