use std::{collections::HashMap, iter::Peekable, str::Chars};

type Attrs = HashMap<String, String>;

static SELF_CLOSING_TAGS: [&'static str; 4] = ["meta", "link", "input", "img"];

#[derive(Debug)]
pub struct NodeData {
    pub tag_name: String,
    pub attributes: Attrs,
}

#[derive(Debug)]
pub struct Node {
    pub children: Vec<Node>,
    pub data: NodeData,
}

impl Node {
    fn new(node_data: NodeData, children: Vec<Node>) -> Self {
        Self {
            data: node_data,
            children,
        }
    }
}

pub struct HTMLParser<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> HTMLParser<'a> {
    pub fn new(source: &'a str) -> HTMLParser {
        Self {
            chars: source.chars().peekable(),
        }
    }
    pub fn parse(&mut self) -> Option<Node> {
        let mut root = Node::new(
            NodeData {
                tag_name: "".to_string(),
                attributes: HashMap::new(),
            },
            Vec::new(),
        );

        // 1. Parse tag name
        self.parse_tag_name(&mut root);

        let tag_name = root.data.tag_name.clone();

        // 2. Parse attributes
        self.parse_attributes(&mut root);

        // do not parse content if it's a self closing tag
        if SELF_CLOSING_TAGS.contains(&tag_name.as_str()) {
            return Some(root);
        }

        // 3. Parse content
        self.parse_content(&mut root);

        Some(root)
    }

    fn parse_tag_name(&mut self, node: &mut Node) {
        let mut tag_name_str = String::new();

        // Collect chars from current pointer until we find an empty space or a closing tag char
        // empty space: <p( )class="">
        // closing tag char: <p(>)
        loop {
            if let Some(next_char) = self.chars.peek() {
                if *next_char == ' ' || *next_char == '>' {
                    break;
                }

                tag_name_str.push(self.chars.next().unwrap());
            } else {
                break;
            }
        }

        // Remove < from the start and / from the end for self closing tags
        // <br/>
        node.data.tag_name = tag_name_str.replace('<', "").replace('/', "");
    }

    fn parse_attributes(&mut self, node: &mut Node) {
        let mut attributes_str = String::new();

        // Collect chars from current pointer until we find a closing tag char
        // closing tag char: <p class="hello"(>)
        loop {
            if let Some(next_char) = self.chars.peek() {
                if *next_char == '>' {
                    // Consume last >
                    self.chars.next().unwrap();
                    break;
                }

                attributes_str.push(self.chars.next().unwrap());
            } else {
                break;
            }
        }
        
        // Trimming, there's always an empty space at the start from previous step (parse tag)
        // <p class="35"></p>
        // parse tag name will consume ane leave: '( )class="asd"'
        attributes_str = attributes_str.trim().to_string();

        // No attributes just return
        if attributes_str.is_empty() {
            return;
        }

        let mut attributes = HashMap::new();

        let attributes_pairs = attributes_str.split(' ').collect::<Vec<&str>>();

        for attr_pair in attributes_pairs {
            let (attr_name, attr_value) = attr_pair
                .split_once('=')
                .ok_or("Error on parsing attribute")
                .unwrap();

            attributes.insert(attr_name.to_string(), attr_value.replace('"', ""));
        }

        node.data.attributes.extend(attributes);
    }

    fn parse_content(&mut self, node: &mut Node) {
        loop {
            if let Some(next_char) = self.chars.peek() {
                // Check if content is another element
                if *next_char == '<' {
                    self.chars.next().unwrap();

                    if let Some('/') = self.chars.peek() {
                        break;
                    }

                    if let Some(child) = self.parse() {
                        node.children.push(child);
                    }
                } else {
                    // Treat content as plain text and skip the closing tag
                    let mut content_str = String::new();

                    loop {
                        if let Some(next_char) = self.chars.peek() {
                            // Tag content is until we find < part of the closing tag --> (<)tagname />
                            if *next_char == '<' {
                                // Consume <
                                self.chars.next().unwrap();
                                break;
                            }

                            content_str.push(self.chars.next().unwrap());
                        } else {
                            break;
                        }
                    }

                    // We create a "text" node for now to represent non-node children
                    // This will contain all CSS / JS / Plan Text
                    let mut text_node = Node {
                        data: NodeData {
                            tag_name: "text".to_string(),
                            attributes: HashMap::new(),
                        },
                        children: Vec::new(),
                    };

                    text_node
                        .data
                        .attributes
                        .insert("content".to_string(), content_str);

                    node.children.push(text_node);

                    // Consum until the end of the tag
                    while let Some(next_char) = self.chars.peek() {
                        if *next_char == '>' {
                            // Consume the '>'
                            self.chars.next().unwrap();
                            break;
                        }
                        self.chars.next().unwrap();
                    }

                    break; // Once we parse non-node content from a node, we just close the loop
                }
            } else {
                break;
            }
        }
    }

    fn is_at_end(&mut self) -> bool {
        self.chars.peek().is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_node() {
        let html = r#"<html data-darkreader-mode="dynamic" data-darkreader-scheme="dark"></html>"#;
        let mut parser = HTMLParser::new(html);
        let node = parser.parse().unwrap();

        assert_eq!(node.data.tag_name, "html");
        assert_eq!(
            node.data.attributes.get("data-darkreader-mode"),
            Some(&"dynamic".to_string())
        );
        assert_eq!(
            node.data.attributes.get("data-darkreader-scheme"),
            Some(&"dark".to_string())
        );
    }

    #[test]
    fn test_parse_text_content() {
        let html = r#"<html data-darkreader-mode="dynamic" data-darkreader-scheme="dark">welcome to my page</html>"#;
        let mut parser = HTMLParser::new(html);
        let node = parser.parse().unwrap();

        let child = node.children.get(0).unwrap();

        assert_eq!(child.data.tag_name, "text".to_string());
        assert_eq!(
            child.data.attributes.get("content"),
            Some(&"welcome to my page".to_string())
        );
    }

    #[test]
    fn test_parse_content() {
        let html = r#"<html data-darkreader-mode="dynamic" data-darkreader-scheme="dark"><h1 class="title-site">Welcome to my page</h1></html>"#;
        let mut parser = HTMLParser::new(html);
        let node = parser.parse().unwrap();
        let h1 = node.children.get(0).unwrap();
        let h1_text_node = h1.children.get(0).unwrap();

        assert_eq!(h1.data.tag_name, "h1".to_string());
        assert_eq!(
            h1.data.attributes.get("class"),
            Some(&"title-site".to_string())
        );
        assert_eq!(
            h1_text_node.data.attributes.get("content"),
            Some(&"Welcome to my page".to_string())
        );
    }

    #[test]
    fn test_parse_sibling_content() {
        let html = r#"<html data-darkreader-mode="dynamic" data-darkreader-scheme="dark"><h1 class="title-site">Welcome to my page</h1><h2 class="subtitle-site">Subtitle content</h2></html>"#;
        let mut parser = HTMLParser::new(html);
        let node = parser.parse().unwrap();
        let h1 = node.children.get(0).unwrap();
        let h1_text_node = h1.children.get(0).unwrap();
        let h2 = node.children.get(1).unwrap();
        let h2_text_node = h2.children.get(0).unwrap();

        assert!(node.children.len() == 2);
        assert_eq!(h1.data.tag_name, "h1".to_string());
        assert_eq!(
            h1.data.attributes.get("class"),
            Some(&"title-site".to_string())
        );
        assert_eq!(
            h1_text_node.data.attributes.get("content"),
            Some(&"Welcome to my page".to_string())
        );
        assert_eq!(
            h2.data.attributes.get("class"),
            Some(&"subtitle-site".to_string())
        );
        assert_eq!(
            h2_text_node.data.attributes.get("content"),
            Some(&"Subtitle content".to_string())
        );
    }

    #[test]
    fn test_parse_meta_tags() {
        let html = r#"<html><head><title>Example Domain</title><meta charset="utf-8"><meta content="text/html; charset=utf-8" http-equiv="Content-type"><meta content="width=device-width,initial-scale=1" name="viewport"></head><body><div><h1>Example Domain</h1><p>This domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission.</p><p><a>More information...</a></p></div></body></html>"#;
        let mut parser = HTMLParser::new(html);
        let node = parser.parse().unwrap();

        println!("{:#?}", node);

        assert!(true);

        // assert!(node.children.len() == 2);
        // assert_eq!(h1.data.tag_name, "h1".to_string());
        // assert_eq!(
        //     h1.data.attributes.get("class"),
        //     Some(&"title-site".to_string())
        // );
        // assert_eq!(
        //     h1_text_node.data.attributes.get("content"),
        //     Some(&"Welcome to my page".to_string())
        // );
        // assert_eq!(
        //     h2.data.attributes.get("class"),
        //     Some(&"subtitle-site".to_string())
        // );
        // assert_eq!(
        //     h2_text_node.data.attributes.get("content"),
        //     Some(&"Subtitle content".to_string())
        // );
    }
}
