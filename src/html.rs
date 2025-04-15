use std::{collections::HashMap, iter::Peekable, str::Chars};

use regex::Regex;

type Attrs = HashMap<String, String>;

static SELF_CLOSING_TAGS: [&'static str; 5] = ["meta", "link", "input", "img", "br"];

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

    #[deprecated]
    pub fn find_text_nodes<'a>(&'a self) -> Vec<&'a Node> {
        let mut collected_nodes = Vec::new();

        self.find_nodes("text", &mut collected_nodes);

        collected_nodes
    }

    #[deprecated]
    pub fn find_nodes<'a>(&'a self, node_type: &str, collected_nodes: &mut Vec<&'a Node>) {
        for child in &self.children {
            if child.data.tag_name == node_type {
                collected_nodes.push(child);
            } else {
                child.find_nodes(node_type, collected_nodes);
            }
        }
    }

    pub fn attr(&self, name: &str) -> &String {
        self.data.attributes.get(name).unwrap()
    }
}

pub struct HTMLParser<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> HTMLParser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.trim().chars().peekable(),
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

        // 2.a. consume white spaces and line feeds before the content
        self.consume_whitespaces();

        // 2.b. do not parse content if it's a self-closing tag
        if SELF_CLOSING_TAGS.contains(&tag_name.as_str()) {
            return Some(root);
        }

        // 4. Parse content
        self.parse_content(&mut root);

        // 5. consume white spaces and line feeds after the content
        self.consume_whitespaces();

        Some(root)
    }

    fn parse_tag_name(&mut self, node: &mut Node) {
        // Collect chars from current pointer until we find an empty space or a closing tag char
        // empty space: <p( )class="">
        // closing tag char: <p(>)
        let tag_name_str = self.read_until(vec![&' ', &'>']);

        // Remove < from the start and / from the end for self-closing tags
        // <br/>
        node.data.tag_name = tag_name_str.replace('<', "").replace('/', "");
    }

    fn parse_attributes(&mut self, node: &mut Node) {
        let attributes_str = self.read_until(vec![&'>']);
        // Consume last >
        self.chars.next().unwrap();

        // No attributes just return
        if attributes_str.is_empty() {
            return;
        }

        let mut attributes = HashMap::new();

        let attributes_pairs = Regex::new(r#"[^\s=]+="[^"]*""#)
            .unwrap()
            .find_iter(attributes_str.as_str())
            .map(|m| m.as_str())
            .collect::<Vec<&str>>();

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

                    // check that we are not in a closing tag or comment instead of an opening one
                    let next_char = self.chars.peek().unwrap().clone();

                    if ['!', '/'].contains(&next_char) {
                        // If we are in a closing tag, consume all the chars until we find a > char
                        self.consume_until(&'>');

                        if next_char == '/' {
                            break; // We break out of the loop since we already parsed child for this element
                        } else {
                            // TODO: remove this from here, find a better place
                            self.consume_whitespaces();
                            continue; // We found a comment, consumed it and keep going
                        }
                    };

                    if let Some(child) = self.parse() {
                        node.children.push(child);
                    }
                } else {
                    // Treat content as plain text and skip the closing tag
                    let content_str = self.read_until(vec![&'<']);

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
                }
            } else {
                break;
            }
        }
    }

    fn read_until(&mut self, chars: Vec<&char>) -> String {
        let mut collected = String::new();

        while let Some(next_char) = self.chars.peek() {
            if chars.contains(&next_char) {
                break;
            }
            collected.push(self.chars.next().unwrap());
        }

        collected
    }

    fn consume_until(&mut self, char: &char) {
        while let Some(_) = self.chars.peek() {
            let consumed = self.chars.next().unwrap();
            if consumed == *char {
                break;
            }
        }
    }

    fn consume_whitespaces(&mut self) {
        while let Some(next_char) = self.chars.peek() {
            if *next_char == ' ' || *next_char == '\t' || *next_char == '\n' {
                self.chars.next().unwrap();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

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
    fn test_parse_style_tags() {
        let html = r#"<html><head><title>Example Domain</title><style class="darkreader darkreader--fallback" media="screen">some attributes</style></head></html>"#;
        let mut parser = HTMLParser::new(html);
        let node = parser.parse().unwrap();

        let head = node.children.get(0).unwrap();

        let style = head.children.get(1).unwrap();

        assert_eq!(style.data.tag_name, "style".to_string());
        assert_eq!(
            style.data.attributes.get("class"),
            Some(&"darkreader darkreader--fallback".to_string())
        );
    }

    #[test]
    fn test_parse_meta_tags() {
        let html = r#"<html><head><title>Example Domain</title><meta charset="utf-8"><meta content="text/html; charset=utf-8" http-equiv="Content-type"><meta content="width=device-width,initial-scale=1" name="viewport"></head><body><div><h1>Example Domain</h1><p>This domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission.</p><p><a>More information...</a></p></div></body></html>"#;
        let mut parser = HTMLParser::new(html);
        let node = parser.parse().unwrap();
        let head = node.children.get(0).unwrap();
        let meta = head.children.get(1).unwrap();

        assert_eq!(meta.data.tag_name, "meta".to_string());
        assert_eq!(
            meta.data.attributes.get("charset"),
            Some(&"utf-8".to_string())
        );
    }

    #[test]
    fn test_parse_long_body_tags() {
        let html = r#"<html data-darkreader-mode="dynamic" data-darkreader-scheme="dark"><head><style class="darkreader darkreader--fallback" media="screen"></style><style class="darkreader darkreader--text" media="screen"></style><style class="darkreader darkreader--invert" media="screen">.captcheck_answer_label>input+img,.d2l-iframe-loading-container,.d2l-navigation-link-image-container,.jfk-bubble.gtx-bubble,a[data-testid=headerMediumLogo]>svg,img.Wirisformula,span#closed_text>img[src^="https://www.gstatic.com/images/branding/googlelogo"],span[data-href^="https://www.hcaptcha.com/"]>#icon{filter:invert(100%) hue-rotate(180deg) contrast(90%)!important}</style><style class="darkreader darkreader--inline" media="screen">[data-darkreader-inline-bgcolor]{background-color:var(--darkreader-inline-bgcolor)!important}[data-darkreader-inline-bgimage]{background-image:var(--darkreader-inline-bgimage)!important}[data-darkreader-inline-border]{border-color:var(--darkreader-inline-border)!important}[data-darkreader-inline-border-bottom]{border-bottom-color:var(--darkreader-inline-border-bottom)!important}[data-darkreader-inline-border-left]{border-left-color:var(--darkreader-inline-border-left)!important}[data-darkreader-inline-border-right]{border-right-color:var(--darkreader-inline-border-right)!important}[data-darkreader-inline-border-top]{border-top-color:var(--darkreader-inline-border-top)!important}[data-darkreader-inline-boxshadow]{box-shadow:var(--darkreader-inline-boxshadow)!important}[data-darkreader-inline-color]{color:var(--darkreader-inline-color)!important}[data-darkreader-inline-fill]{fill:var(--darkreader-inline-fill)!important}[data-darkreader-inline-stroke]{stroke:var(--darkreader-inline-stroke)!important}[data-darkreader-inline-outline]{outline-color:var(--darkreader-inline-outline)!important}[data-darkreader-inline-stopcolor]{stop-color:var(--darkreader-inline-stopcolor)!important}[data-darkreader-inline-bg]{background:var(--darkreader-inline-bg)!important}[data-darkreader-inline-border-short]{border:var(--darkreader-inline-border-short)!important}[data-darkreader-inline-border-bottom-short]{border-bottom:var(--darkreader-inline-border-bottom-short)!important}[data-darkreader-inline-border-left-short]{border-left:var(--darkreader-inline-border-left-short)!important}[data-darkreader-inline-border-right-short]{border-right:var(--darkreader-inline-border-right-short)!important}[data-darkreader-inline-border-top-short]{border-top:var(--darkreader-inline-border-top-short)!important}[data-darkreader-inline-invert]{filter:invert(100%) hue-rotate(180deg)}</style><style class="darkreader darkreader--variables" media="screen">:root{--darkreader-neutral-background:var(--darkreader-background-ffffff, #181a1b);--darkreader-neutral-text:var(--darkreader-text-000000, #e8e6e3);--darkreader-selection-background:var(--darkreader-background-0060d4, #004daa);--darkreader-selection-text:var(--darkreader-text-ffffff, #e8e6e3)}</style><style class="darkreader darkreader--root-vars" media="screen"></style><style class="darkreader darkreader--user-agent" media="screen">html{color-scheme:dark!important}iframe{color-scheme:dark!important}body,html{background-color:var(--darkreader-background-ffffff,#181a1b)}body,html{border-color:var(--darkreader-border-4c4c4c,#736b5e);color:var(--darkreader-text-000000,#e8e6e3)}a{color:var(--darkreader-text-0040ff,#3391ff)}table{border-color:var(--darkreader-border-808080,#545b5e)}mark{color:var(--darkreader-text-000000,#e8e6e3)}::placeholder{color:var(--darkreader-text-a9a9a9,#b2aba1)}input:-webkit-autofill,select:-webkit-autofill,textarea:-webkit-autofill{background-color:var(--darkreader-background-faffbd,#404400)!important;color:var(--darkreader-text-000000,#e8e6e3)!important}::selection{background-color:var(--darkreader-background-0060d4,#004daa)!important;color:var(--darkreader-text-ffffff,#e8e6e3)!important}::-moz-selection{background-color:var(--darkreader-background-0060d4,#004daa)!important;color:var(--darkreader-text-ffffff,#e8e6e3)!important}</style><title>Example Domain</title><meta charset="utf-8"><meta http-equiv="Content-type" content="text/html; charset=utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><style type="text/css">body{background-color:#f0f0f2;margin:0;padding:0;font-family:-apple-system,system-ui,BlinkMacSystemFont,"Segoe UI","Open Sans","Helvetica Neue",Helvetica,Arial,sans-serif}div{width:600px;margin:5em auto;padding:2em;background-color:#fdfdff;border-radius:.5em;box-shadow:2px 3px 7px 2px rgba(0,0,0,.02)}a:link,a:visited{color:#38488f;text-decoration:none}@media (max-width:700px){div{margin:0 auto;width:auto}}</style><style class="darkreader darkreader--sync" media="screen"></style><meta name="darkreader" content="67eee74fa8317ce9478ac4c4612115ec"><style class="darkreader darkreader--override" media="screen">.vimvixen-hint{background-color:var(--darkreader-background-ffd76e,#684b00)!important;border-color:var(--darkreader-background-c59d00,#9e7e00)!important;color:var(--darkreader-text-302505,#d7d4cf)!important}#vimvixen-console-frame{color-scheme:light!important}::placeholder{opacity:.5!important}#edge-translate-panel-body,.MuiTypography-body1,.nfe-quote-text{color:var(--darkreader-neutral-text)!important}gr-main-header{background-color:var(--darkreader-background-add8e6,#1b4958)!important}.tou-1b6i2ox,.tou-lnqlqk,.tou-mignzq,.tou-z65h9k{background-color:var(--darkreader-neutral-background)!important}.tou-75mvi{background-color:var(--darkreader-background-cfecf5,#0f3a47)!important}.tou-17ezmgn,.tou-1b8t2us,.tou-1frrtv8,.tou-1lpmd9d,.tou-1w3fhi0,.tou-py7lfi,.tou-ta9e87{background-color:var(--darkreader-background-f5f5f5,#1e2021)!important}.tou-uknfeu{background-color:var(--darkreader-background-faedda,#432c09)!important}.tou-6i3zyv{background-color:var(--darkreader-background-85c3d8,#245d70)!important}div.mermaid-viewer-control-panel .btn{background-color:var(--darkreader-neutral-background);fill:var(--darkreader-neutral-text)}svg g rect.er{fill:var(--darkreader-neutral-background)!important}svg g rect.er.entityBox{fill:var(--darkreader-neutral-background)!important}svg g rect.er.attributeBoxOdd{fill:var(--darkreader-neutral-background)!important}svg g rect.er.attributeBoxEven{fill:var(--darkreader-selection-background);fill-opacity:.8!important}svg rect.er.relationshipLabelBox{fill:var(--darkreader-neutral-background)!important}svg g g.nodes polygon,svg g g.nodes rect{fill:var(--darkreader-neutral-background)!important}svg g rect.task{fill:var(--darkreader-selection-background)!important}svg line.messageLine0,svg line.messageLine1{stroke:var(--darkreader-neutral-text)!important}div.mermaid .actor{fill:var(--darkreader-neutral-background)!important}mitid-authenticators-code-app>.code-app-container{background-color:#fff!important;padding-top:1rem}iframe#unpaywall[src$="unpaywall.html"]{color-scheme:light!important}select option{background-color:var(--darkreader-neutral-background)!important}body#tumblr{--darkreader-bg--secondary-accent:31,32,34!important;--darkreader-bg--white:23,23,23!important;--darkreader-text--black:228,224,218!important}:host{--d2l-border-color:var(--darkreader-bg--d2l-color-gypsum)!important;--d2l-button-icon-background-color-hover:var(--darkreader-bg--d2l-color-gypsum)!important;--d2l-color-ferrite:var(--darkreader-neutral-text)!important;--d2l-color-sylvite:var(--darkreader-bg--d2l-color-sylvite)!important;--d2l-dropdown-background-color:var(--darkreader-neutral-background)!important;--d2l-dropdown-border-color:var(--darkreader-border--d2l-color-mica)!important;--d2l-input-backgroud-color:var(--darkreader-neutral-background)!important;--d2l-menu-border-color:var(--darkreader-bg--d2l-color-gypsum)!important;--d2l-tooltip-background-color:var(--darkreader-neutral-background)!important;--d2l-tooltip-border-color:var(--darkreader-bg--d2l-color-gypsum)!important}:host([_floating]) .d2l-floating-buttons-container{background-color:var(--darkreader-neutral-background)!important;border-top-color:var(--darkreader-border--d2l-color-mica)!important;opacity:.88!important}d2l-card{background:var(--darkreader-neutral-background)!important;border-color:var(--darkreader-border--d2l-color-gypsum)!important}d2l-dropdown-content>div,d2l-menu-item{background-color:var(--darkreader-neutral-background)!important;border-radius:10px!important}d2l-empty-state-simple{border-color:var(--darkreader-bg--d2l-color-gypsum)!important}.d2l-button-filter>ul>li>a.vui-button{border-color:var(--darkreader-border--d2l-color-mica)!important}.d2l-label-text:has(.d2l-button-subtle-content):active,.d2l-label-text:has(.d2l-button-subtle-content):focus,.d2l-label-text:has(.d2l-button-subtle-content):hover{background-color:var(--darkreader-bg--d2l-color-gypsum)!important}.d2l-navigation-centerer{color:inherit!important}.d2l-tabs-layout{border-color:var(--darkreader-border--d2l-color-gypsum)!important}.d2l-calendar-date,.d2l-htmleditor-container,.d2l-input{background-color:var(--darkreader-neutral-background)!important}.d2l-collapsible-panel{border:1px solid var(--darkreader-border--d2l-color-mica)!important;border-radius:.4rem!important}.d2l-collapsible-panel-divider{border-bottom:1px solid var(--darkreader-border--d2l-color-mica)!important}.d2l-w2d-flex{border-bottom:2px solid var(--darkreader-border--d2l-color-mica)!important}.d2l-collapsible-panel scrolled,.d2l-collapsible-panel-header,.d2l-w2d-collection-fixed{background-color:var(--darkreader-neutral-background)!important}.d2l-loading-spinner-bg{fill:var(--darkreader-bg--d2l-color-gypsum)!important}.d2l-loading-spinner-bg-stroke{stroke:var(--darkreader-border--d2l-color-mica)!important}.d2l-loading-spinner-wrapper svg circle,.d2l-loading-spinner-wrapper svg path{fill:var(--darkreader-neutral-background)!important}</style></head><body><div><h1>Example Domain</h1><p>This domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission.</p><p><a href="https://www.iana.org/domains/example">More information...</a></p></div></body></html>"#;
        let mut parser = HTMLParser::new(html);
        let node = parser.parse().unwrap();
        let body = node.children.get(1).unwrap();
        let div = body.children.get(0).unwrap();
        let h1 = div.children.get(0).unwrap();

        assert_eq!(h1.data.tag_name, "h1".to_string());

        let text = h1.children.get(0).unwrap();

        assert_eq!(
            text.data.attributes.get("content"),
            Some(&"Example Domain".to_string())
        );
    }

    #[test]
    fn test_search_text_nodes() {
        let html = r#"<html><head><title>Example Domain</title><meta charset="utf-8"><meta content="text/html; charset=utf-8" http-equiv="Content-type"><meta content="width=device-width,initial-scale=1" name="viewport"></head><body><div><h1>Example Domain</h1><p>This domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission.</p><p><a>More information...</a></p></div></body></html>"#;
        let mut parser = HTMLParser::new(html);
        let root = parser.parse().unwrap();
        let text_nodes = root.find_text_nodes();

        assert_eq!(text_nodes.len(), 4);
        assert_eq!(text_nodes[0].attr("content"), "Example Domain");
        assert_eq!(text_nodes[1].attr("content"), "Example Domain");
        assert_eq!(text_nodes[2].attr("content"), "This domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission.");
        assert_eq!(text_nodes[3].attr("content"), "More information...");
    }

    #[test]
    fn test_consume_whitespaces() {
        let html = r#"<
            html><head></head></html>"#;
        let mut parser = HTMLParser::new(html);

        assert_eq!(*parser.chars.peek().unwrap(), '<');
        // Consume the (<)
        parser.chars.next();
        // Consume all white spaces
        parser.consume_whitespaces();
        assert_eq!(parser.chars.next(), Some('h'));
    }

    #[test]
    fn test_consume_until() {
        let html = r#"
        <html>
            <head></head>
        </html>"#;
        let mut parser = HTMLParser::new(html);
        parser.consume_until(&'<');

        assert_eq!(parser.chars.next(), Some('h'));
    }

    #[test]
    fn test_consume_read_until() {
        let html = r#"hello world</>"#;
        let mut parser = HTMLParser::new(html);
        let collected = parser.read_until(vec![&'<']);

        assert_eq!(collected, "hello world".to_string());
        assert_eq!(parser.chars.next(), Some('<'));
    }

    #[test]
    fn test_ignore_whitespaces() {
        let html = r#"
        <html data-darkreader-mode="dynamic" data-darkreader-scheme="dark">
            <h1 class="title-site">Welcome to my page</h1>
            <h2 class="subtitle-site">Subtitle content</h2>
        </html>
        "#;
        let mut parser = HTMLParser::new(html);
        let node = parser.parse().unwrap();

        println!("{:#?}", node);

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
    fn test_self_closing_tags() {
        let html = r#"
            <blockquote>
            一派白虹起，千寻雪浪飞。<br>
            海风吹不断，江月照还依。<br>
            冷气分青嶂，余流润翠微。<br>
            潺盢名瀑布，真似挂帘帷。<br>
            </blockquote>
            "#;
        let mut parser = HTMLParser::new(html);
        let node = parser.parse().unwrap();

        assert_eq!(node.children.len(), 8);
    }

    #[test]
    fn test_nested_spans() {
        let html = r#"
            <blockquote>
            一派白虹起，<span>千寻雪浪飞。</span><br>
            海风吹不断，江月照还依。<br>
            <!-- Content originally taken from https://www.zggdwx.com/xiyou.html -->
            冷气分青嶂，余流润翠微。<br>
            潺盢名瀑布，真似挂帘帷。<br>
            </blockquote>
            "#;
        let mut parser = HTMLParser::new(html);
        let node = parser.parse().unwrap();

        assert_eq!(node.children.len(), 9);
    }

    #[test]
    fn test_full_text() {
        let html_str = read_to_string("server/web.html").unwrap();
        let mut parser = HTMLParser::new(&html_str);

        let root = parser.parse().unwrap();
        let nodes = root.find_text_nodes();

        assert_eq!(nodes.len(), 83);
    }
}
