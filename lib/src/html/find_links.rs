use html5ever::{local_name, LocalName};
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

impl LinkTag {
    pub fn attr_name(&self) -> LocalName {
        match self {
            LinkTag::A => local_name!("href"),
            LinkTag::Img => local_name!("src"),
            LinkTag::Link => local_name!("href"),
        }
    }
}

impl Link {
    pub fn from_node(node: NodeRef) -> Option<Self> {
        if let NodeData::Element(data) = node.data() {
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

        None
    }

    pub fn link(&self) -> Option<String> {
        if let NodeData::Element(data) = self.node.data() {
            let attr_name = self.tag.attr_name();

            let attrs = data.attributes.borrow();

            if let Some(attr_value) = attrs.get(attr_name) {
                return Some(attr_value.to_owned());
            }
        }

        None
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
        if let NodeData::Element(data) = self.node.data() {
            let attr_name = self.tag.attr_name();

            let mut attrs = data.attributes.borrow_mut();

            let mut original_value: Option<String> = None;

            if let Some(attr_value) = attrs.get_mut(&attr_name) {
                original_value = Some(attr_value.clone());

                attr_value.clear();
                attr_value.push_str(with);
            }

            if let Some(original_value) = original_value {
                attrs.insert(format!("data-original-{}", &attr_name), original_value);
            }
        }
    }
}

pub fn find_links(node: NodeRef) -> Vec<Link> {
    find_elements(node, |_| true).filter_map(Link::from_node).collect()
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

    const HTML_REPLACE_LINKS: &'static str = r###"
<html>
    <head></head>
    <body>
        <div>
            <a href="a"></a>
            <img src="images/file.png">
            <link href="css/style.css" rel="stylesheet">
        </div>
    </body>
</html>
"###;

    const HTML_REPLACE_LINKS_EXPECTED_RESULT: &'static str = r###"
<html>
    <head></head>
    <body>
        <div>
            <a data-original-href="a" href="A"></a>
            <img data-original-src="images/file.png" src="IMAGES/FILE.PNG">
            <link data-original-href="css/style.css" href="CSS/STYLE.CSS" rel="stylesheet">
        </div>
    </body>
</html>
"###;

    #[test]
    fn can_replace_links() {
        let node = parse_string(&normalize(HTML_REPLACE_LINKS));

        for link in find_links(node.clone()).iter_mut() {
            if let Some(value) = link.link() {
                link.replace(&value.to_uppercase());
            }
        }

        let result = to_string(node);

        assert_eq!(result, normalize(HTML_REPLACE_LINKS_EXPECTED_RESULT));
    }
}
