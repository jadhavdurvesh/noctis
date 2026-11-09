//! A deliberately small, independently implemented HTML parser.
//! This is an experimental subset, not an HTML Living Standard implementation.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Document(Vec<Node>),
    Element {
        name: String,
        attributes: Vec<Attribute>,
        children: Vec<Node>,
    },
    Text(String),
    Comment(String),
    Doctype(String),
}

impl Node {
    pub fn children(&self) -> &[Node] {
        match self {
            Self::Document(children) | Self::Element { children, .. } => children,
            _ => &[],
        }
    }

    pub fn attribute(&self, name: &str) -> Option<&str> {
        let Self::Element { attributes, .. } = self else { return None };
        attributes.iter()
            .find(|attr| attr.name.eq_ignore_ascii_case(name))
            .and_then(|attr| attr.value.as_deref())
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        let Self::Element { attributes, .. } = self else { return false };
        attributes.iter().any(|attr| attr.name.eq_ignore_ascii_case(name))
    }

    /// Stable diagnostic tree output; not an HTML serializer.
    pub fn dump(&self) -> String {
        fn visit(node: &Node, depth: usize, out: &mut String) {
            let indent = "  ".repeat(depth);
            match node {
                Node::Document(children) => {
                    out.push_str(&format!("{indent}#document\n"));
                    for child in children { visit(child, depth + 1, out); }
                }
                Node::Element { name, attributes, children } => {
                    out.push_str(&format!("{indent}<{name}"));
                    for attr in attributes {
                        match &attr.value {
                            Some(value) => out.push_str(&format!(" {}={value:?}", attr.name)),
                            None => out.push_str(&format!(" {}", attr.name)),
                        }
                    }
                    out.push_str(">\n");
                    for child in children { visit(child, depth + 1, out); }
                }
                Node::Text(text) => out.push_str(&format!("{indent}#text {text:?}\n")),
                Node::Comment(text) => out.push_str(&format!("{indent}#comment {text:?}\n")),
                Node::Doctype(name) => out.push_str(&format!("{indent}<!doctype {name}>\n")),
            }
        }

        let mut output = String::new();
        visit(self, 0, &mut output);
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    StartTag { name: String, attributes: Vec<Attribute>, self_closing: bool },
    EndTag(String),
    Text(String),
    Comment(String),
    Doctype(String),
}

fn is_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':')
}

fn read_name(chars: &[char], mut index: usize) -> (String, usize) {
    let start = index;
    while index < chars.len() && is_name_char(chars[index]) { index += 1; }
    (chars[start..index].iter().collect::<String>().to_ascii_lowercase(), index)
}

fn skip_space(chars: &[char], mut index: usize) -> usize {
    while index < chars.len() && chars[index].is_whitespace() { index += 1; }
    index
}

fn tokenize(source: &str) -> Vec<Token> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] != '<' {
            let start = i;
            while i < chars.len() && chars[i] != '<' { i += 1; }
            tokens.push(Token::Text(chars[start..i].iter().collect()));
            continue;
        }

        if chars[i..].starts_with(&['<', '!', '-', '-']) {
            i += 4;
            let start = i;
            while i + 2 < chars.len() && !chars[i..].starts_with(&['-', '-', '>']) { i += 1; }
            let comment: String = chars[start..i].iter().collect();
            i = (i + 3).min(chars.len());
            tokens.push(Token::Comment(comment));
            continue;
        }

        if chars.len() - i >= 9
            && chars[i..i + 9].iter().collect::<String>().eq_ignore_ascii_case("<!doctype")
        {
            i += 9;
            i = skip_space(&chars, i);
            let (name, next) = read_name(&chars, i);
            i = next;
            while i < chars.len() && chars[i] != '>' { i += 1; }
            i = (i + 1).min(chars.len());
            tokens.push(Token::Doctype(if name.is_empty() { "html".into() } else { name }));
            continue;
        }

        if i + 1 < chars.len() && chars[i + 1] == '/' {
            i = skip_space(&chars, i + 2);
            let (name, next) = read_name(&chars, i);
            i = next;
            while i < chars.len() && chars[i] != '>' { i += 1; }
            i = (i + 1).min(chars.len());
            if !name.is_empty() { tokens.push(Token::EndTag(name)); }
            continue;
        }

        if i + 1 < chars.len() && is_name_char(chars[i + 1]) {
            let (name, next) = read_name(&chars, i + 1);
            i = next;
            let mut attributes = Vec::new();
            let mut self_closing = false;

            loop {
                i = skip_space(&chars, i);
                if i >= chars.len() { break; }
                if chars[i] == '>' { i += 1; break; }
                if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '>' {
                    self_closing = true;
                    i += 2;
                    break;
                }

                let attr_start = i;
                while i < chars.len()
                    && !chars[i].is_whitespace()
                    && !matches!(chars[i], '=' | '>' | '/')
                {
                    i += 1;
                }
                if i == attr_start {
                    // Consume malformed input to guarantee progress.
                    i += 1;
                    continue;
                }

                let attr_name: String = chars[attr_start..i].iter().collect::<String>().to_ascii_lowercase();
                i = skip_space(&chars, i);
                let value = if i < chars.len() && chars[i] == '=' {
                    i = skip_space(&chars, i + 1);
                    if i < chars.len() && matches!(chars[i], '"' | '\'') {
                        let quote = chars[i];
                        i += 1;
                        let value_start = i;
                        while i < chars.len() && chars[i] != quote { i += 1; }
                        let value: String = chars[value_start..i].iter().collect();
                        if i < chars.len() { i += 1; }
                        Some(value)
                    } else {
                        let value_start = i;
                        while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '>' { i += 1; }
                        Some(chars[value_start..i].iter().collect())
                    }
                } else {
                    None
                };

                // Keep the first occurrence of a duplicate attribute name.
                if !attributes.iter().any(|attr: &Attribute| attr.name == attr_name) {
                    attributes.push(Attribute { name: attr_name, value });
                }
            }

            tokens.push(Token::StartTag { name, attributes, self_closing });
            continue;
        }

        // A '<' that cannot begin a recognized token is ordinary text.
        tokens.push(Token::Text("<".into()));
        i += 1;
    }

    tokens
}

fn is_void(name: &str) -> bool {
    matches!(name, "area" | "base" | "br" | "col" | "embed" | "hr" | "img"
        | "input" | "link" | "meta" | "param" | "source" | "track" | "wbr")
}

fn node_at_path<'a>(root: &'a Node, path: &[usize]) -> &'a Node {
    let mut current = root;
    for &index in path {
        current = match current {
            Node::Document(children) | Node::Element { children, .. } => &children[index],
            _ => unreachable!("only container nodes appear in a path"),
        };
    }
    current
}

fn children_at_path_mut<'a>(root: &'a mut Node, path: &[usize]) -> &'a mut Vec<Node> {
    let mut current = root;
    for &index in path {
        current = match current {
            Node::Document(children) | Node::Element { children, .. } => &mut children[index],
            _ => unreachable!("only container nodes appear in a path"),
        };
    }
    match current {
        Node::Document(children) | Node::Element { children, .. } => children,
        _ => unreachable!("path must end at a container"),
    }
}

/// Parse a limited HTML subset. Error recovery and tree construction are not HTML5-complete.
pub fn parse(source: &str) -> Node {
    let mut root = Node::Document(Vec::new());
    let mut open_elements: Vec<Vec<usize>> = Vec::new();

    for token in tokenize(source) {
        let path = open_elements.last().cloned().unwrap_or_default();
        match token {
            Token::Text(text) => children_at_path_mut(&mut root, &path).push(Node::Text(text)),
            Token::Comment(text) => children_at_path_mut(&mut root, &path).push(Node::Comment(text)),
            Token::Doctype(name) => children_at_path_mut(&mut root, &path).push(Node::Doctype(name)),
            Token::StartTag { name, attributes, self_closing } => {
                let children = children_at_path_mut(&mut root, &path);
                let index = children.len();
                children.push(Node::Element { name: name.clone(), attributes, children: Vec::new() });
                if !self_closing && !is_void(&name) {
                    let mut child_path = path;
                    child_path.push(index);
                    open_elements.push(child_path);
                }
            }
            Token::EndTag(name) => {
                if let Some(position) = open_elements.iter().rposition(|candidate| {
                    matches!(node_at_path(&root, candidate), Node::Element { name: open, .. } if open == &name)
                }) {
                    open_elements.truncate(position);
                }
            }
        }
    }

    root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_nested_elements_and_text() {
        let tree = parse("<html><body><h1>Hello</h1><p>World</p></body></html>");
        let dump = tree.dump();
        assert!(dump.contains("<html>"));
        assert!(dump.contains("<h1>"));
        assert!(dump.contains("#text \"Hello\""));
        assert!(dump.contains("<p>"));
    }

    #[test]
    fn parses_attributes_and_boolean_attributes() {
        let tree = parse("<a href='https://example.test' disabled>link</a>");
        let link = &tree.children()[0];
        assert_eq!(link.attribute("href"), Some("https://example.test"));
        assert!(link.has_attribute("disabled"));
        assert!(link.dump().contains("disabled"));
    }

    #[test]
    fn void_elements_do_not_become_open_ancestors() {
        let tree = parse("<p>a<br>b</p>");
        let dump = tree.dump();
        assert!(dump.contains("<br>"));
        assert!(dump.contains("#text \"b\""));
    }

    #[test]
    fn handles_comments_doctype_and_unicode() {
        let tree = parse("<!DOCTYPE html><!--note--><p>नमस्ते</p>");
        let dump = tree.dump();
        assert!(dump.contains("<!doctype html>"));
        assert!(dump.contains("#comment \"note\""));
        assert!(dump.contains("नमस्ते"));
    }

    #[test]
    fn malformed_less_than_is_preserved_as_text() {
        let tree = parse("2 < 3");
        assert!(tree.dump().contains("#text \"<\""));
    }

    #[test]
    fn closes_to_the_matching_open_element() {
        let tree = parse("<a><b><c>x</b>y</a>");
        let dump = tree.dump();
        assert!(dump.contains("<b>"));
        assert!(dump.contains("#text \"y\""));
    }
}
