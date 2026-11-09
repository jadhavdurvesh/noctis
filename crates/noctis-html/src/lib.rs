//! NOCTIS's independently implemented HTML tokenizer, fragment tree builder,
//! and lightweight DOM primitives.
//!
//! This is an explicitly incomplete HTML subset. It is not an HTML Living
//! Standard implementation and must not be treated as safe for arbitrary web
//! content until the parser and browser security model mature.

pub mod document;
pub use document::{DomDocument, DomError, DomNode, DomNodeId, DomNodeKind};

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
            if node
                .name()
                .is_some_and(|node_name| node_name.eq_ignore_ascii_case(name))
            {
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

        fn serialize(node: &Node, output: &mut String, raw_text_parent: bool) {
            match node {
                Node::Document(children) => {
                    for child in children {
                        serialize(child, output, false);
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
                        let is_raw_text = raw_text_element(name);
                        for child in children {
                            serialize(child, output, is_raw_text);
                        }
                        output.push_str("</");
                        output.push_str(name);
                        output.push('>');
                    }
                }
                Node::Text(text) => {
                    if raw_text_parent {
                        output.push_str(text);
                    } else {
                        output.push_str(&escape_text(text, false));
                    }
                }
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
        serialize(self, &mut output, false);
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

const NAMED_ENTITY_DATA: &str = include_str!("named_entities.tsv");

fn named_entity_value(name: &str) -> Option<&'static str> {
    use std::sync::OnceLock;

    static INDEX: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
    let index = INDEX.get_or_init(|| {
        NAMED_ENTITY_DATA
            .lines()
            .filter_map(|line| line.split_once('\t'))
            .collect()
    });
    index
        .binary_search_by(|(entity_name, _)| entity_name.cmp(&name))
        .ok()
        .map(|position| index[position].1)
}

fn append_named_entity(output: &mut String, codepoints: &str) {
    for codepoint in codepoints.split(',') {
        if let Ok(value) = u32::from_str_radix(codepoint, 16) {
            output.push(char::from_u32(value).unwrap_or('\u{fffd}'));
        }
    }
}

fn decode_character_references_with_context(input: &str, in_attribute: bool) -> String {
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
                match parsed {
                    Ok(value) => output.push(numeric_character_reference(value)),
                    Err(_) => output.push('\u{fffd}'),
                }
                index = cursor;
                continue;
            }
        } else {
            while cursor < chars.len()
                && cursor - start <= 32
                && (chars[cursor].is_ascii_alphanumeric() || chars[cursor] == ';')
            {
                cursor += 1;
            }

            let max_end = cursor;
            let mut matched = None;
            for end in (start + 1..=max_end).rev() {
                let candidate: String = chars[start + 1..end].iter().collect();
                if let Some(value) = named_entity_value(&candidate) {
                    let has_semicolon = candidate.ends_with(';');
                    let next = chars.get(end).copied();
                    let ambiguous_attribute_match = in_attribute
                        && !has_semicolon
                        && next.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '=');
                    if ambiguous_attribute_match {
                        matched = None;
                        break;
                    }
                    matched = Some((value, end));
                    break;
                }
            }

            if let Some((value, end)) = matched {
                append_named_entity(&mut output, value);
                index = end;
                continue;
            }
        }

        output.push('&');
        index += 1;
    }

    output
}

/// Decode the complete HTML named-character-reference table and numeric references.
pub fn decode_character_references(input: &str) -> String {
    decode_character_references_with_context(input, false)
}

/// Decode character references using the HTML attribute ambiguous-ampersand rule.
pub fn decode_character_references_in_attribute(input: &str) -> String {
    decode_character_references_with_context(input, true)
}

fn numeric_character_reference(value: u32) -> char {
    const WINDOWS_1252: [u32; 32] = [
        0x20ac, 0x81, 0x201a, 0x192, 0x201e, 0x2026, 0x2020, 0x2021, 0x2c6, 0x2030, 0x160, 0x2039,
        0x152, 0x8d, 0x17d, 0x8f, 0x90, 0x2018, 0x2019, 0x201c, 0x201d, 0x2022, 0x2013, 0x2014,
        0x2dc, 0x2122, 0x161, 0x203a, 0x153, 0x9d, 0x17e, 0x178,
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
                match parsed {
                    Ok(value) => output.push(numeric_character_reference(value)),
                    Err(_) => output.push('\u{fffd}'),
                }
                index = cursor;
                continue;
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
                        Some(decode_character_references_in_attribute(&raw_value))
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
pub fn parse_fragment(source: &str) -> Node {
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

/// Parse a complete HTML document into an `html/head/body` document skeleton.
///
/// This implements common document setup and head/body routing for the currently
/// supported subset. It is not the full HTML Living Standard insertion-mode algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocumentInsertionMode {
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
}

/// Parse a complete HTML document into an `html/head/body` document skeleton.
///
/// This implements a deliberately scoped set of document insertion modes and
/// common head/body recovery. It is not the complete HTML Living Standard
/// tree-construction algorithm (table foster parenting, adoption agency,
/// templates, foreign-content integration, and document mode are still pending).
pub fn parse_document(source: &str) -> Node {
    use std::collections::VecDeque;

    let fragment = parse_fragment(source);
    let Node::Document(fragment_children) = fragment else {
        unreachable!("fragment parser always returns a document node");
    };

    let mut html = Node::Element {
        name: "html".into(),
        attributes: Vec::new(),
        children: Vec::new(),
    };
    let mut head = Node::Element {
        name: "head".into(),
        attributes: Vec::new(),
        children: Vec::new(),
    };
    let mut body = Node::Element {
        name: "body".into(),
        attributes: Vec::new(),
        children: Vec::new(),
    };
    let mut html_attributes = Vec::new();
    let mut head_attributes = Vec::new();
    let mut body_attributes = Vec::new();
    let mut doctype = None;

    // Unwrap an explicit html element while preserving source order.
    let mut pending = VecDeque::new();
    for child in fragment_children {
        match child {
            Node::Doctype(name) if doctype.is_none() => doctype = Some(name),
            Node::Element {
                name,
                attributes,
                children,
            } if name == "html" => {
                merge_attributes(&mut html_attributes, attributes);
                pending.extend(children);
            }
            other => pending.push_back(other),
        }
    }

    let mut mode = DocumentInsertionMode::BeforeHtml;
    while let Some(node) = pending.pop_front() {
        if matches!(node, Node::Doctype(_)) {
            continue;
        }

        match node {
            Node::Element {
                name,
                attributes,
                children,
            } if name == "head" => {
                merge_attributes(&mut head_attributes, attributes);
                if matches!(
                    mode,
                    DocumentInsertionMode::BeforeHtml
                        | DocumentInsertionMode::BeforeHead
                        | DocumentInsertionMode::InHead
                ) {
                    append_nodes(&mut head, children);
                    mode = DocumentInsertionMode::AfterHead;
                } else {
                    // Duplicate or misplaced head wrappers are ignored as wrappers.
                    pending.extend(children);
                }
            }
            Node::Element {
                name,
                attributes,
                children,
            } if name == "body" => {
                merge_attributes(&mut body_attributes, attributes);
                append_nodes(&mut body, children);
                mode = DocumentInsertionMode::InBody;
            }
            other => {
                if mode == DocumentInsertionMode::BeforeHtml {
                    mode = DocumentInsertionMode::BeforeHead;
                    pending.push_front(other);
                    continue;
                }

                match mode {
                    DocumentInsertionMode::BeforeHtml => unreachable!(),
                    DocumentInsertionMode::BeforeHead => {
                        if is_head_content(&other)
                            || is_whitespace_text(&other)
                            || matches!(other, Node::Comment(_))
                        {
                            head.append_child(other).expect("head is a container");
                            mode = DocumentInsertionMode::InHead;
                        } else {
                            body.append_child(other).expect("body is a container");
                            mode = DocumentInsertionMode::InBody;
                        }
                    }
                    DocumentInsertionMode::InHead => {
                        if is_head_content(&other) || is_whitespace_text(&other) || matches!(other, Node::Comment(_)) {
                            head.append_child(other).expect("head is a container");
                        } else {
                            // Leaving the head implicitly creates the body.
                            mode = DocumentInsertionMode::AfterHead;
                            pending.push_front(other);
                        }
                    }
                    DocumentInsertionMode::AfterHead => {
                        body.append_child(other).expect("body is a container");
                        mode = DocumentInsertionMode::InBody;
                    }
                    DocumentInsertionMode::InBody => {
                        body.append_child(other).expect("body is a container");
                    }
                }
            }
        }
    }

    if let Node::Element { attributes, .. } = &mut html {
        *attributes = html_attributes;
    }
    if let Node::Element { attributes, .. } = &mut head {
        *attributes = head_attributes;
    }
    if let Node::Element { attributes, .. } = &mut body {
        *attributes = body_attributes;
    }

    if let Node::Element { children, .. } = &mut html {
        children.push(head);
        children.push(body);
    }

    let mut output = Vec::new();
    if let Some(name) = doctype {
        output.push(Node::Doctype(name));
    }
    output.push(html);
    Node::Document(output)
}

fn merge_attributes(target: &mut Vec<Attribute>, incoming: Vec<Attribute>) {
    for attribute in incoming {
        if !target
            .iter()
            .any(|existing| existing.name == attribute.name)
        {
            target.push(attribute);
        }
    }
}

fn append_nodes(parent: &mut Node, children: Vec<Node>) {
    for child in children {
        parent
            .append_child(child)
            .expect("document containers accept children");
    }
}

fn is_whitespace_text(node: &Node) -> bool {
    matches!(node, Node::Text(text) if text.chars().all(char::is_whitespace))
}

fn is_head_content(node: &Node) -> bool {
    matches!(
        node.name(),
        Some(
            "base"
                | "basefont"
                | "bgsound"
                | "link"
                | "meta"
                | "title"
                | "noscript"
                | "noframes"
                | "style"
                | "script"
                | "template"
        )
    )
}

fn find_doctype(source: &str) -> Option<String> {
    let lower = source.to_ascii_lowercase();
    let start = lower.find("<!doctype")? + "<!doctype".len();
    let tail = &source[start..];
    let name_start = tail
        .find(|ch: char| !ch.is_whitespace())
        .unwrap_or(tail.len());
    let tail = &tail[name_start..];
    let name_end = tail
        .find(|ch: char| !is_name_char(ch))
        .unwrap_or(tail.len());
    let name = tail[..name_end].trim();
    Some(if name.is_empty() {
        "html".into()
    } else {
        name.to_ascii_lowercase()
    })
}

/// Backwards-compatible alias for fragment parsing.
pub fn parse(source: &str) -> Node {
    parse_fragment(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_nested_elements_and_text() {
        let tree = parse_fragment("<html><body><h1>Hello</h1><p>World</p></body></html>");
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
        let duplicate = parse_fragment("<a href='first' href='second'>x</a>");
        assert_eq!(duplicate.children()[0].attribute("href"), Some("first"));
    }

    #[test]
    fn void_elements_do_not_become_open_ancestors() {
        let tree = parse("<p>a<br>b</p>");
        let paragraph = &tree.children()[0];
        assert_eq!(paragraph.text_content(), "ab");
        assert_eq!(paragraph.children().len(), 3);
        assert_eq!(paragraph.children()[1].name(), Some("br"));
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
        assert_eq!(
            script.to_html(),
            "<script>if (a < b) value = '<tag>';</script>"
        );
        assert_eq!(tree.elements_by_tag_name("p").len(), 1);
    }

    #[test]
    fn handles_rcdata_in_title_and_textarea() {
        let tree =
            parse_fragment("<title>A &amp; B <titleish></title><textarea>&lt;ok&gt;</textarea>");
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
        assert_eq!(decode_character_references("&#999999999999999999999;"), "�");
        assert_eq!(decode_character_references("&unknown;"), "&unknown;");
    }

    #[test]
    fn applies_common_implied_end_tags() {
        let list = parse_fragment("<ul><li>one<li>two</ul>");
        let items = list.elements_by_tag_name("li");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].text_content(), "one");
        assert_eq!(items[1].text_content(), "two");

        let paragraphs = parse_fragment("<p>one<div>two</div>");
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
        assert!(main.append_child(removed).is_ok());
        assert_eq!(
            tree.to_html(),
            "<main class=\"content\"><p id=\"target\">Hello &amp; welcome</p></main>"
        );
    }

    #[test]
    fn handles_comments_doctypes_unicode_and_unterminated_comments() {
        let tree = parse("<!DOCTYPE html><!--note--><p>नमस्ते</p>");
        assert!(tree.dump().contains("<!doctype html>"));
        assert!(tree.dump().contains("#comment \"note\""));
        assert_eq!(tree.children().last().unwrap().text_content(), "नमस्ते");

        let unterminated = parse_fragment("<!-- unfinished");
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
