//! NOCTIS's independently implemented HTML tokenizer, fragment tree builder,
//! and lightweight DOM primitives.
//!
//! This is an explicitly incomplete HTML subset. It is not an HTML Living
//! Standard implementation and must not be treated as safe for arbitrary web
//! content until the parser and browser security model mature.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    /// None represents a boolean attribute; Some("") is an empty value.
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

    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Element { name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn attributes(&self) -> &[Attribute] {
        match self {
            Self::Element { attributes, .. } => attributes,
            _ => &[],
        }
    }

    pub fn attribute(&self, name: &str) -> Option<&str> {
        let Self::Element { attributes, .. } = self else {
            return None;
        };
        attributes
            .iter()
            .find(|attribute| attribute.name.eq_ignore_ascii_case(name))
            .and_then(|attribute| attribute.value.as_deref())
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        let Self::Element { attributes, .. } = self else {
            return false;
        };
        attributes
            .iter()
            .any(|attribute| attribute.name.eq_ignore_ascii_case(name))
    }

    /// Set or replace an attribute. Returns false for non-element nodes.
    pub fn set_attribute(&mut self, name: &str, value: Option<&str>) -> bool {
        let Self::Element { attributes, .. } = self else {
            return false;
        };
        let normalized_name = name.to_ascii_lowercase();
        if let Some(attribute) = attributes
            .iter_mut()
            .find(|attribute| attribute.name == normalized_name)
        {
            attribute.value = value.map(str::to_owned);
        } else {
            attributes.push(Attribute {
                name: normalized_name,
                value: value.map(str::to_owned),
            });
        }
        true
    }

    /// Remove an attribute and return it, if present.
    pub fn remove_attribute(&mut self, name: &str) -> Option<Attribute> {
        let Self::Element { attributes, .. } = self else {
            return None;
        };
        let index = attributes
            .iter()
            .position(|attribute| attribute.name.eq_ignore_ascii_case(name))?;
        Some(attributes.remove(index))
    }

    /// Append a child to a document or element. Returns the child on failure.
    pub fn append_child(&mut self, child: Node) -> Result<(), Node> {
        match self {
            Self::Document(children) | Self::Element { children, .. } => {
                children.push(child);
                Ok(())
            }
            _ => Err(child),
        }
    }

    /// Remove and return a child at the requested index.
    pub fn remove_child(&mut self, index: usize) -> Option<Node> {
        match self {
            Self::Document(children) | Self::Element { children, .. } if index < children.len() => {
                Some(children.remove(index))
            }
            _ => None,
        }
    }

    /// Find the first element with the requested id, including this node.
    pub fn find_by_id(&self, id: &str) -> Option<&Node> {
        if self.attribute("id") == Some(id) {
            return Some(self);
        }
        self.children()
            .iter()
            .find_map(|child| child.find_by_id(id))
    }

    /// Collect matching elements in tree order, including this node if it matches.
    pub fn elements_by_tag_name<'a>(&'a self, tag_name: &str) -> Vec<&'a Node> {
        fn collect<'a>(node: &'a Node, name: &str, matches: &mut Vec<&'a Node>) {
            if node.name().is_some_and(|node_name| node_name.eq_ignore_ascii_case(name)) {
                matches.push(node);
            }
            for child in node.children() {
                collect(child, name, matches);
            }
        }

        let mut matches = Vec::new();
        collect(self, tag_name, &mut matches);
        matches
    }

    /// Concatenate descendant text nodes in document order.
    pub fn text_content(&self) -> String {
        fn collect(node: &Node, output: &mut String) {
            match node {
                Node::Text(text) => output.push_str(text),
                Node::Document(children) | Node::Element { children, .. } => {
                    for child in children {
                        collect(child, output);
                    }
                }
                Node::Comment(_) | Node::Doctype(_) => {}
            }
        }

        let mut output = String::new();
        collect(self, &mut output);
        output
    }

    /// Serialize this subset of the DOM as HTML.
    pub fn to_html(&self) -> String {
        fn escape_text(input: &str, attribute: bool) -> String {
            let mut output = String::with_capacity(input.len());
            for character in input.chars() {
                match character {
                    '&' => output.push_str("&amp;"),
                    '<' => output.push_str("&lt;"),
                    '>' if !attribute => output.push_str("&gt;"),
                    '"' if attribute => output.push_str("&quot;"),
                    _ => output.push(character),
                }
            }
            output
        }

        fn serialize(node: &Node, output: &mut String) {
            match node {
                Node::Document(children) => {
                    for child in children {
                        serialize(child, output);
                    }
                }
                Node::Element {
                    name,
                    attributes,
                    children,
                } => {
                    output.push('<');
                    output.push_str(name);
                    for attribute in attributes {
                        output.push(' ');
                        output.push_str(&attribute.name);
                        if let Some(value) = &attribute.value {
                            output.push_str("=\"");
                            output.push_str(&escape_text(value, true));
                            output.push('"');
                        }
                    }
                    output.push('>');
                    if !is_void_element(name) {
                        for child in children {
                            serialize(child, output);
                        }
                        output.push_str("</");
                        output.push_str(name);
                        output.push('>');
                    }
                }
                Node::Text(text) => output.push_str(&escape_text(text, false)),
                Node::Comment(comment) => {
                    output.push_str("<!--");
                    output.push_str(comment);
                    output.push_str("-->");
                }
                Node::Doctype(name) => {
                    output.push_str("<!doctype ");
                    output.push_str(name);
                    output.push('>');
                }
            }
        }

        let mut output = String::new();
        serialize(self, &mut output);
        output
    }

    /// Stable diagnostic tree output; not an HTML serializer.
    pub fn dump(&self) -> String {
        fn visit(node: &Node, depth: usize, output: &mut String) {
            let indent = "  ".repeat(depth);
            match node {
                Node::Document(children) => {
                    output.push_str(&format!("{indent}#document\n"));
                    for child in children {
                        visit(child, depth + 1, output);
                    }
                }
                Node::Element {
                    name,
                    attributes,
                    children,
                } => {
                    output.push_str(&format!("{indent}<{name}"));
                    for attribute in attributes {
                        match &attribute.value {
                            Some(value) => {
                                output.push_str(&format!(" {}={value:?}", attribute.name))
                            }
                            None => output.push_str(&format!(" {}", attribute.name)),
                        }
                    }
                    output.push_str(">\n");
                    for child in children {
                        visit(child, depth + 1, output);
                    }
                }
                Node::Text(text) => {
                    output.push_str(&format!("{indent}#text {text:?}\n"));
                }
                Node::Comment(text) => {
                    output.push_str(&format!("{indent}#comment {text:?}\n"));
                }
                Node::Doctype(name) => {
                    output.push_str(&format!("{indent}<!doctype {name}>\n"));
                }
            }
        }

        let mut output = String::new();
        visit(self, 0, &mut output);
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    StartTag {
        name: String,
        attributes: Vec<Attribute>,
        self_closing: bool,
    },
    EndTag(String),
    Text(String),
    Comment(String),
    Doctype(String),
}

fn is_name_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ':')
}

fn read_name(chars: &[char], mut index: usize) -> (String, usize) {
    let start = index;
    while index < chars.len() && is_name_char(chars[index]) {
        index += 1;
    }
    (
        chars[start..index]
            .iter()
            .collect::<String>()
            .to_ascii_lowercase(),
        index,
    )
}

fn skip_space(chars: &[char], mut index: usize) -> usize {
    while index < chars.len() && chars[index].is_whitespace() {
        index += 1;
    }
    index
}

fn starts_with_ascii_case_insensitive(chars: &[char], index: usize, pattern: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    index + pattern_chars.len() <= chars.len()
        && chars[index..index + pattern_chars.len()]
            .iter()
            .zip(pattern_chars.iter())
            .all(|(left, right)| left.eq_ignore_ascii_case(right))
}

fn named_character_reference(name: &str) -> Option<char> {
    Some(match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{00a0}',
        "copy" => '\u{00a9}',
        "reg" => '\u{00ae}',
        "trade" => '\u{2122}',
        "hellip" => '\u{2026}',
        "mdash" => '\u{2014}',
        "ndash" => '\u{2013}',
        "euro" => '\u{20ac}',
        "pound" => '\u{00a3}',
        "cent" => '\u{00a2}',
        "yen" => '\u{00a5}',
        "sect" => '\u{00a7}',
        "para" => '\u{00b6}',
        "middot" => '\u{00b7}',
        "laquo" => '\u{00ab}',
        "raquo" => '\u{00bb}',
        "times" => '\u{00d7}',
        "divide" => '\u{00f7}',
        "plusmn" => '\u{00b1}',
        "deg" => '\u{00b0}',
        "micro" => '\u{00b5}',
        "bull" => '\u{2022}',
        "rsquo" => '\u{2019}',
        "lsquo" => '\u{2018}',
        "rdquo" => '\u{201d}',
        "ldquo" => '\u{201c}',
        _ => return None,
    })
}

fn numeric_character_reference(value: u32) -> char {
    const WINDOWS_1252: [u32; 32] = [
        0x20ac, 0x81, 0x201a, 0x192, 0x201e, 0x2026, 0x2020, 0x2021, 0x2c6, 0x2030, 0x160,
        0x2039, 0x152, 0x8d, 0x17d, 0x8f, 0x90, 0x2018, 0x2019, 0x201c, 0x201d, 0x2022, 0x2013,
        0x2014, 0x2dc, 0x2122, 0x161, 0x203a, 0x153, 0x9d, 0x17e, 0x178,
    ];

    let mapped = if (0x80..=0x9f).contains(&value) {
        WINDOWS_1252[(value - 0x80) as usize]
    } else {
        value
    };
    if mapped == 0 || (0xd800..=0xdfff).contains(&mapped) {
        '\u{fffd}'
    } else {
        char::from_u32(mapped).unwrap_or('\u{fffd}')
    }
}

/// Decode a practical subset of named and numeric HTML character references.
/// Named references in this initial implementation require a trailing semicolon.
pub fn decode_character_references(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len());
    let mut index = 0;

    while index < chars.len() {
        if chars[index] != '&' {
            output.push(chars[index]);
            index += 1;
            continue;
        }

        let start = index;
        let mut cursor = index + 1;
        if cursor < chars.len() && chars[cursor] == '#' {
            cursor += 1;
            let is_hex = cursor < chars.len() && matches!(chars[cursor], 'x' | 'X');
            if is_hex {
                cursor += 1;
            }
            let digits_start = cursor;
            while cursor < chars.len()
                && if is_hex {
                    chars[cursor].is_ascii_hexdigit()
                } else {
                    chars[cursor].is_ascii_digit()
                }
            {
                cursor += 1;
            }
            if cursor > digits_start {
                let digits: String = chars[digits_start..cursor].iter().collect();
                let parsed = if is_hex {
                    u32::from_str_radix(&digits, 16)
                } else {
                    digits.parse::<u32>()
                };
                if cursor < chars.len() && chars[cursor] == ';' {
                    cursor += 1;
                }
                if let Ok(value) = parsed {
                    output.push(numeric_character_reference(value));
                    index = cursor;
                    continue;
                }
            }
        } else {
            while cursor < chars.len()
                && cursor - start <= 32
                && chars[cursor].is_ascii_alphanumeric()
            {
                cursor += 1;
            }
            if cursor > start + 1 && cursor < chars.len() && chars[cursor] == ';' {
                let name: String = chars[start + 1..cursor].iter().collect();
                if let Some(decoded) = named_character_reference(&name) {
                    output.push(decoded);
                    index = cursor + 1;
                    continue;
                }
            }
        }

        output.push('&');
        index += 1;
    }

    output
}

fn is_void_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn raw_text_element(name: &str) -> bool {
    matches!(
        name,
        "script" | "style" | "xmp" | "iframe" | "noembed" | "noframes" | "plaintext"
    )
}

fn rcdata_element(name: &str) -> bool {
    matches!(name, "title" | "textarea")
}

fn find_raw_text_end(chars: &[char], from: usize, tag_name: &str) -> Option<usize> {
    let mut index = from;
    while index + tag_name.len() + 2 <= chars.len() {
        if chars[index] == '<'
            && index + 1 < chars.len()
            && chars[index + 1] == '/'
            && starts_with_ascii_case_insensitive(chars, index + 2, tag_name)
        {
            let boundary = index + 2 + tag_name.len();
            if boundary == chars.len()
                || chars[boundary].is_whitespace()
                || matches!(chars[boundary], '>' | '/')
            {
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

fn tokenize(source: &str) -> Vec<Token> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;
    let mut raw_tag: Option<String> = None;

    while index < chars.len() {
        if let Some(tag_name) = raw_tag.take() {
            if tag_name == "plaintext" {
                tokens.push(Token::Text(chars[index..].iter().collect()));
                break;
            }
            if let Some(end) = find_raw_text_end(&chars, index, &tag_name) {
                let text: String = chars[index..end].iter().collect();
                tokens.push(Token::Text(if rcdata_element(&tag_name) {
                    decode_character_references(&text)
                } else {
                    text
                }));
                index = end;
            } else {
                let text: String = chars[index..].iter().collect();
                tokens.push(Token::Text(if rcdata_element(&tag_name) {
                    decode_character_references(&text)
                } else {
                    text
                }));
                break;
            }
        }

        if chars[index] != '<' {
            let start = index;
            while index < chars.len() && chars[index] != '<' {
                index += 1;
            }
            let text: String = chars[start..index].iter().collect();
            tokens.push(Token::Text(decode_character_references(&text)));
            continue;
        }

        if index + 3 < chars.len() && chars[index..].starts_with(&['<', '!', '-', '-']) {
            index += 4;
            let start = index;
            while index + 2 < chars.len() && !chars[index..].starts_with(&['-', '-', '>']) {
                index += 1;
            }
            let comment_end = if index + 2 < chars.len() {
                let end = index;
                index += 3;
                end
            } else {
                index = chars.len();
                chars.len()
            };
            let comment: String = chars[start..comment_end].iter().collect();
            tokens.push(Token::Comment(comment));
            continue;
        }

        if starts_with_ascii_case_insensitive(&chars, index, "<!doctype") {
            index += 9;
            index = skip_space(&chars, index);
            let (name, next) = read_name(&chars, index);
            index = next;
            while index < chars.len() && chars[index] != '>' {
                index += 1;
            }
            index = (index + 1).min(chars.len());
            tokens.push(Token::Doctype(if name.is_empty() {
                "html".into()
            } else {
                name
            }));
            continue;
        }

        if index + 1 < chars.len() && chars[index + 1] == '/' {
            index = skip_space(&chars, index + 2);
            let (name, next) = read_name(&chars, index);
            index = next;
            while index < chars.len() && chars[index] != '>' {
                index += 1;
            }
            index = (index + 1).min(chars.len());
            if !name.is_empty() {
                tokens.push(Token::EndTag(name));
            }
            continue;
        }

        if index + 1 < chars.len() && is_name_char(chars[index + 1]) {
            let (name, next) = read_name(&chars, index + 1);
            index = next;
            let mut attributes = Vec::new();
            let mut self_closing = false;

            loop {
                index = skip_space(&chars, index);
                if index >= chars.len() {
                    break;
                }
                if chars[index] == '>' {
                    index += 1;
                    break;
                }
                if chars[index] == '/' && index + 1 < chars.len() && chars[index + 1] == '>' {
                    self_closing = true;
                    index += 2;
                    break;
                }

                let attribute_start = index;
                while index < chars.len()
                    && !chars[index].is_whitespace()
                    && !matches!(chars[index], '=' | '>')
                {
                    index += 1;
                }
                if index == attribute_start {
                    index += 1;
                    continue;
                }

                let attribute_name: String = chars[attribute_start..index]
                    .iter()
                    .collect::<String>()
                    .to_ascii_lowercase();
                index = skip_space(&chars, index);

                let value = if index < chars.len() && chars[index] == '=' {
                    index = skip_space(&chars, index + 1);
                    if index < chars.len() && matches!(chars[index], '"' | '\'') {
                        let quote = chars[index];
                        index += 1;
                        let value_start = index;
                        while index < chars.len() && chars[index] != quote {
                            index += 1;
                        }
                        let raw_value: String = chars[value_start..index].iter().collect();
                        if index < chars.len() {
                            index += 1;
                        }
                        Some(decode_character_references(&raw_value))
                    } else {
                        let value_start = index;
                        while index < chars.len()
                            && !chars[index].is_whitespace()
                            && chars[index] != '>'
                        {
                            index += 1;
                        }
                        let raw_value: String = chars[value_start..index].iter().collect();
                        Some(decode_character_references(&raw_value))
                    }
                } else {
                    None
                };

                // HTML keeps the first occurrence of a duplicate attribute.
                if !attributes
                    .iter()
                    .any(|attribute: &Attribute| attribute.name == attribute_name)
                {
                    attributes.push(Attribute {
                        name: attribute_name,
                        value,
                    });
                }
            }

            if (raw_text_element(&name) || rcdata_element(&name)) && !self_closing {
                raw_tag = Some(name.clone());
            }
            tokens.push(Token::StartTag {
                name,
                attributes,
                self_closing,
            });
            continue;
        }

        tokens.push(Token::Text("<".into()));
        index += 1;
    }

    tokens
}

#[derive(Debug, Clone)]
struct OpenElement {
    path: Vec<usize>,
    name: String,
}

fn node_at_path<'a>(root: &'a Node, path: &[usize]) -> &'a Node {
    let mut current = root;
    for &index in path {
        current = match current {
            Node::Document(children) | Node::Element { children, .. } => &children[index],
            _ => unreachable!("only container nodes appear in an element path"),
        };
    }
    current
}

fn children_at_path_mut<'a>(root: &'a mut Node, path: &[usize]) -> &'a mut Vec<Node> {
    let mut current = root;
    for &index in path {
        current = match current {
            Node::Document(children) | Node::Element { children, .. } => &mut children[index],
            _ => unreachable!("only container nodes appear in an element path"),
        };
    }
    match current {
        Node::Document(children) | Node::Element { children, .. } => children,
        _ => unreachable!("an element path must end at a container"),
    }
}

fn close_open_element(stack: &mut Vec<OpenElement>, name: &str) {
    if let Some(position) = stack.iter().rposition(|element| element.name == name) {
        stack.truncate(position);
    }
}

fn close_one_of(stack: &mut Vec<OpenElement>, names: &[&str]) {
    if let Some(position) = stack
        .iter()
        .rposition(|element| names.contains(&element.name.as_str()))
    {
        stack.truncate(position);
    }
}

fn apply_implied_end_tags(stack: &mut Vec<OpenElement>, incoming: &str) {
    match incoming {
        "li" => close_open_element(stack, "li"),
        "dt" | "dd" => close_one_of(stack, &["dt", "dd"]),
        "option" => close_open_element(stack, "option"),
        "optgroup" => {
            close_open_element(stack, "option");
            close_open_element(stack, "optgroup");
        }
        "tr" => close_open_element(stack, "tr"),
        "td" | "th" => close_one_of(stack, &["td", "th"]),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            close_one_of(stack, &["h1", "h2", "h3", "h4", "h5", "h6"]);
        }
        "address" | "article" | "aside" | "blockquote" | "div" | "dl" | "fieldset" | "footer"
        | "form" | "hgroup" | "hr" | "main" | "menu" | "nav" | "ol" | "p" | "pre" | "section"
        | "table" | "ul" => close_open_element(stack, "p"),
        _ => {}
    }
}

/// Parse an HTML fragment into a document node.
///
/// This supports a useful subset of HTML syntax and error recovery. It does not
/// implement the full HTML Living Standard tree-construction algorithm.
pub fn parse(source: &str) -> Node {
    let mut root = Node::Document(Vec::new());
    let mut open_elements: Vec<OpenElement> = Vec::new();

    for token in tokenize(source) {
        match token {
            Token::Text(text) => {
                let path = open_elements
                    .last()
                    .map_or(&[][..], |element| element.path.as_slice());
                children_at_path_mut(&mut root, path).push(Node::Text(text));
            }
            Token::Comment(comment) => {
                let path = open_elements
                    .last()
                    .map_or(&[][..], |element| element.path.as_slice());
                children_at_path_mut(&mut root, path).push(Node::Comment(comment));
            }
            Token::Doctype(name) => {
                let path = open_elements
                    .last()
                    .map_or(&[][..], |element| element.path.as_slice());
                children_at_path_mut(&mut root, path).push(Node::Doctype(name));
            }
            Token::StartTag {
                name,
                attributes,
                self_closing,
            } => {
                apply_implied_end_tags(&mut open_elements, &name);
                let path = open_elements
                    .last()
                    .map_or_else(Vec::new, |element| element.path.clone());
                let children = children_at_path_mut(&mut root, &path);
                let child_index = children.len();
                children.push(Node::Element {
                    name: name.clone(),
                    attributes,
                    children: Vec::new(),
                });

                if !self_closing && !is_void_element(&name) {
                    let mut child_path = path;
                    child_path.push(child_index);
                    open_elements.push(OpenElement {
                        path: child_path,
                        name,
                    });
                }
            }
            Token::EndTag(name) => {
                if name == "p" && !open_elements.iter().any(|element| element.name == "p") {
                    let path = open_elements
                        .last()
                        .map_or_else(Vec::new, |element| element.path.clone());
                    let children = children_at_path_mut(&mut root, &path);
                    let child_index = children.len();
                    children.push(Node::Element {
                        name: "p".into(),
                        attributes: Vec::new(),
                        children: Vec::new(),
                    });
                    let mut child_path = path;
                    child_path.push(child_index);
                    open_elements.push(OpenElement {
                        path: child_path,
                        name: "p".into(),
                    });
                }
                if let Some(position) = open_elements
                    .iter()
                    .rposition(|element| element.name == name)
                {
                    open_elements.truncate(position);
                }
            }
        }
    }

    debug_assert!(open_elements
        .iter()
        .all(|element| matches!(node_at_path(&root, &element.path), Node::Element { .. })));
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
    fn preserves_first_duplicate_attribute_and_parses_unquoted_values() {
        let tree = parse("<img src=https://example.test/a.png alt=sample/>");
        let image = &tree.children()[0];
        assert_eq!(image.attribute("src"), Some("https://example.test/a.png"));
        assert_eq!(image.attribute("alt"), Some("sample/"));
        assert!(image.has_attribute("src"));
        let duplicate = parse("<a href='first' href='second'>x</a>");
        assert_eq!(duplicate.children()[0].attribute("href"), Some("first"));
    }

    #[test]
    fn void_elements_do_not_become_open_ancestors() {
        let tree = parse("<p>a<br>b</p>");
        let paragraph = &tree.children()[0];
        assert_eq!(paragraph.text_content(), "ab");
        assert_eq!(paragraph.children().len(), 2);
    }

    #[test]
    fn decodes_named_and_numeric_character_references() {
        let tree = parse("<p title='A &amp; B'>&lt; &amp; &#65; &#x1F680; &copy;</p>");
        let paragraph = &tree.children()[0];
        assert_eq!(paragraph.attribute("title"), Some("A & B"));
        assert_eq!(paragraph.text_content(), "< & A 🚀 ©");
    }

    #[test]
    fn preserves_script_and_style_raw_text() {
        let tree = parse("<script>if (a < b) value = '<tag>';</script><p>after</p>");
        let script = &tree.children()[0];
        assert_eq!(script.text_content(), "if (a < b) value = '<tag>';");
        assert_eq!(tree.elements_by_tag_name("p").len(), 1);
    }

    #[test]
    fn handles_rcdata_in_title_and_textarea() {
        let tree = parse("<title>A &amp; B <titleish></title><textarea>&lt;ok&gt;</textarea>");
        let title = &tree.elements_by_tag_name("title")[0];
        let textarea = &tree.elements_by_tag_name("textarea")[0];
        assert_eq!(title.text_content(), "A & B <titleish>");
        assert_eq!(textarea.text_content(), "<ok>");
    }

    #[test]
    fn replaces_invalid_numeric_character_references() {
        assert_eq!(decode_character_references("&#0;"), "�");
        assert_eq!(decode_character_references("&#xD800;"), "�");
        assert_eq!(decode_character_references("&#x110000;"), "�");
        assert_eq!(decode_character_references("&unknown;"), "&unknown;");
    }

    #[test]
    fn applies_common_implied_end_tags() {
        let list = parse("<ul><li>one<li>two</ul>");
        let items = list.elements_by_tag_name("li");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].text_content(), "one");
        assert_eq!(items[1].text_content(), "two");

        let paragraphs = parse("<p>one<div>two</div>");
        assert_eq!(paragraphs.elements_by_tag_name("p").len(), 1);
        assert_eq!(paragraphs.elements_by_tag_name("div").len(), 1);
    }

    #[test]
    fn supports_dom_queries_mutation_and_serialization() {
        let mut tree = parse("<main><p id='target'>Hello &amp; welcome</p></main>");
        let paragraph = tree.find_by_id("target").expect("id lookup");
        assert_eq!(paragraph.text_content(), "Hello & welcome");
        assert_eq!(tree.elements_by_tag_name("p").len(), 1);

        let main = match &mut tree {
            Node::Document(children) => &mut children[0],
            _ => unreachable!(),
        };
        assert!(main.set_attribute("class", Some("content")));
        assert_eq!(main.attribute("class"), Some("content"));
        let removed = main.remove_child(0).expect("remove paragraph");
        assert_eq!(removed.name(), Some("p"));
        assert!(main.children().is_empty());
        assert_eq!(tree.to_html(), "<main class=\"content\"></main>");
    }

    #[test]
    fn handles_comments_doctypes_unicode_and_unterminated_comments() {
        let tree = parse("<!DOCTYPE html><!--note--><p>नमस्ते</p>");
        assert!(tree.dump().contains("<!doctype html>"));
        assert!(tree.dump().contains("#comment \"note\""));
        assert_eq!(tree.children().last().unwrap().text_content(), "नमस्ते");

        let unterminated = parse("<!-- unfinished");
        assert_eq!(unterminated.children().len(), 1);
        assert_eq!(
            unterminated.children()[0],
            Node::Comment(" unfinished".into())
        );
    }

    #[test]
    fn malformed_less_than_is_preserved_as_text() {
        let tree = parse("2 < 3");
        assert_eq!(tree.text_content(), "2 < 3");
    }

    #[test]
    fn closes_to_the_matching_open_element() {
        let tree = parse("<a><b><c>x</b>y</a>");
        let dump = tree.dump();
        assert!(dump.contains("<b>"));
        assert!(dump.contains("#text \"y\""));
    }
}
