//! Readable document extraction for the NOCTIS terminal browser.
//!
//! This is a deliberately small browser presentation layer. It uses NOCTIS's
//! own HTML tree and does not execute JavaScript or delegate rendering to a
//! third-party browser engine.

use noctis_html::{parse_document, Node};
use noctis_net::resolve_reference;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Heading(u8),
    Paragraph,
    ListItem,
    Preformatted,
    Quote,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentBlock {
    pub kind: BlockKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageLink {
    pub number: usize,
    pub text: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentView {
    pub title: String,
    pub blocks: Vec<ContentBlock>,
    pub links: Vec<PageLink>,
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn find_first<'a>(node: &'a Node, tag: &str) -> Option<&'a Node> {
    if node
        .name()
        .is_some_and(|name| name.eq_ignore_ascii_case(tag))
    {
        return Some(node);
    }
    node.children()
        .iter()
        .find_map(|child| find_first(child, tag))
}

fn block_kind(tag: &str) -> Option<BlockKind> {
    match tag {
        "h1" => Some(BlockKind::Heading(1)),
        "h2" => Some(BlockKind::Heading(2)),
        "h3" => Some(BlockKind::Heading(3)),
        "h4" => Some(BlockKind::Heading(4)),
        "h5" => Some(BlockKind::Heading(5)),
        "h6" => Some(BlockKind::Heading(6)),
        "p" => Some(BlockKind::Paragraph),
        "li" => Some(BlockKind::ListItem),
        "pre" => Some(BlockKind::Preformatted),
        "blockquote" => Some(BlockKind::Quote),
        "address" | "figcaption" | "dt" | "dd" => Some(BlockKind::Other),
        _ => None,
    }
}

fn collect_blocks(node: &Node, output: &mut Vec<ContentBlock>) {
    match node {
        Node::Document(children) => {
            for child in children {
                collect_blocks(child, output);
            }
        }
        Node::Element { name, children, .. } => {
            if matches!(
                name.as_str(),
                "script" | "style" | "noscript" | "template" | "svg"
            ) {
                return;
            }
            if let Some(kind) = block_kind(name) {
                let text = collapse_whitespace(&node.text_content());
                if !text.is_empty() {
                    output.push(ContentBlock { kind, text });
                }
                return;
            }
            if name == "img" {
                if let Some(alt) = node
                    .attribute("alt")
                    .map(collapse_whitespace)
                    .filter(|s| !s.is_empty())
                {
                    output.push(ContentBlock {
                        kind: BlockKind::Other,
                        text: format!("[Image: {alt}]"),
                    });
                }
                return;
            }
            for child in children {
                collect_blocks(child, output);
            }
        }
        Node::Text(_) | Node::Comment(_) | Node::Doctype(_) => {}
    }
}

fn collect_links(node: &Node, base_url: &str, output: &mut Vec<PageLink>) {
    match node {
        Node::Document(children) | Node::Element { children, .. } => {
            if let Node::Element { name, .. } = node {
                if matches!(
                    name.as_str(),
                    "script" | "style" | "noscript" | "template" | "svg"
                ) {
                    return;
                }
                if name == "a" {
                    if let Some(href) = node.attribute("href") {
                        if let Ok(url) = resolve_reference(base_url, href) {
                            let text = collapse_whitespace(&node.text_content());
                            let text = if text.is_empty() {
                                href.trim().to_owned()
                            } else {
                                text
                            };
                            output.push(PageLink {
                                number: output.len() + 1,
                                text,
                                url,
                            });
                        }
                    }
                }
            }
            for child in children {
                collect_links(child, base_url, output);
            }
        }
        Node::Text(_) | Node::Comment(_) | Node::Doctype(_) => {}
    }
}

impl DocumentView {
    /// Parse fetched HTML using NOCTIS and extract readable blocks and links.
    pub fn from_html(source: &str, base_url: &str) -> Self {
        let document = parse_document(source);
        let title = find_first(&document, "title")
            .map(|node| collapse_whitespace(&node.text_content()))
            .filter(|title| !title.is_empty())
            .unwrap_or_else(|| base_url.to_owned());

        let content_root = find_first(&document, "body").unwrap_or(&document);
        let mut blocks = Vec::new();
        collect_blocks(content_root, &mut blocks);

        if blocks.is_empty() {
            let fallback = collapse_whitespace(&content_root.text_content());
            if !fallback.is_empty() {
                blocks.push(ContentBlock {
                    kind: BlockKind::Other,
                    text: fallback,
                });
            }
        }

        let mut links = Vec::new();
        collect_links(content_root, base_url, &mut links);

        Self {
            title,
            blocks,
            links,
        }
    }

    /// Build a readable view of a plain-text response.
    pub fn from_plain_text(source: &str, title: &str) -> Self {
        let blocks = source
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| ContentBlock {
                kind: BlockKind::Paragraph,
                text: line.to_owned(),
            })
            .collect();
        Self {
            title: title.to_owned(),
            blocks,
            links: Vec::new(),
        }
    }
}

/// Normalize an address-bar entry. Bare hostnames default to HTTPS.
pub fn normalize_address(input: &str) -> Result<String, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("enter a URL or hostname".to_owned());
    }
    let candidate = if input.contains("://") {
        input.to_owned()
    } else {
        format!("https://{input}")
    };
    noctis_net::parse_web_url(&candidate)
        .map(|url| url.to_string())
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_title_headings_paragraphs_images_and_links() {
        let html = r#"
            <!doctype html>
            <html><head><title>Example page</title><script>do not show</script></head>
            <body>
              <h1>Welcome to NOCTIS</h1>
              <p>A <a href="/about">small browser</a> built from scratch.</p>
              <ul><li>First item</li><li>Second item</li></ul>
              <img src="/logo.png" alt="NOCTIS logo">
              <script>secret()</script><style>.x { color:red }</style>
            </body></html>
        "#;
        let view = DocumentView::from_html(html, "https://example.com/start");
        assert_eq!(view.title, "Example page");
        assert!(view
            .blocks
            .iter()
            .any(|block| block.kind == BlockKind::Heading(1)));
        assert!(view
            .blocks
            .iter()
            .any(|block| block.text.contains("small browser")));
        assert!(view
            .blocks
            .iter()
            .any(|block| block.text == "[Image: NOCTIS logo]"));
        assert!(!view
            .blocks
            .iter()
            .any(|block| block.text.contains("secret")));
        assert_eq!(view.links.len(), 1);
        assert_eq!(view.links[0].number, 1);
        assert_eq!(view.links[0].url, "https://example.com/about");
    }

    #[test]
    fn resolves_links_relative_to_the_final_document_url() {
        let view = DocumentView::from_html(
            r#"<body><a href="../next?q=1">Next page</a><a href="mailto:test@example.com">Mail</a></body>"#,
            "https://example.com/dir/current",
        );
        assert_eq!(view.links.len(), 1);
        assert_eq!(view.links[0].url, "https://example.com/next?q=1");
    }

    #[test]
    fn plain_text_fallback_keeps_nonempty_lines() {
        let view = DocumentView::from_plain_text("first line\n\nsecond line", "Text page");
        assert_eq!(view.title, "Text page");
        assert_eq!(view.blocks.len(), 2);
    }

    #[test]
    fn bare_hostnames_default_to_https_and_non_web_schemes_are_rejected() {
        assert_eq!(
            normalize_address("example.com").unwrap(),
            "https://example.com/"
        );
        assert!(normalize_address("file:///etc/passwd").is_err());
    }
}
