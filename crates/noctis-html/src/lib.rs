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
                && cursor - start <= 33
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

fn preprocess_input(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut characters = source.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '\r' => {
                if characters.peek() == Some(&'\n') {
                    characters.next();
                }
                output.push('\n');
            }
            '\0' => output.push('\u{fffd}'),
            _ => output.push(character),
        }
    }
    output
}

fn tokenize(source: &str) -> Vec<Token> {
    let preprocessed = preprocess_input(source);
    let chars: Vec<char> = preprocessed.chars().collect();
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
            if chars.get(index) == Some(&'>') {
                index += 1;
                tokens.push(Token::Comment(String::new()));
                continue;
            }
            if index + 1 < chars.len() && chars[index..].starts_with(&['-', '>']) {
                index += 2;
                tokens.push(Token::Comment(String::new()));
                continue;
            }
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

        // The HTML tokenizer treats other <! ... > and <? ... > sequences as
        // bogus comments rather than literal text.
        if index + 1 < chars.len() && chars[index] == '<' && matches!(chars[index + 1], '!' | '?') {
            index += 2;
            let start = index;
            while index < chars.len() && chars[index] != '>' {
                index += 1;
            }
            let comment: String = chars[start..index].iter().collect();
            if index < chars.len() {
                index += 1;
            }
            tokens.push(Token::Comment(comment));
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
                        Some(decode_character_references_in_attribute(&raw_value))
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

            if raw_text_element(&name) || rcdata_element(&name) {
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

fn is_formatting_element(name: &str) -> bool {
    matches!(
        name,
        "a" | "b"
            | "big"
            | "code"
            | "em"
            | "font"
            | "i"
            | "s"
            | "small"
            | "strike"
            | "strong"
            | "tt"
            | "u"
    )
}

fn insert_foster_node(
    root: &mut Node,
    stack: &mut [OpenElement],
    node: Node,
) -> Option<Vec<usize>> {
    let table = stack.iter().rposition(|element| element.name == "table")?;
    let table_path = stack[table].path.clone();
    let table_index = *table_path.last()?;
    let parent_path = table_path[..table_path.len() - 1].to_vec();

    children_at_path_mut(root, &parent_path).insert(table_index, node);
    for element in stack.iter_mut() {
        if element.path.len() > parent_path.len()
            && element.path[..parent_path.len()] == parent_path
            && element.path[parent_path.len()] >= table_index
        {
            element.path[parent_path.len()] += 1;
        }
    }

    let mut foster_path = parent_path;
    foster_path.push(table_index);
    Some(foster_path)
}

fn append_open_element(
    root: &mut Node,
    stack: &mut Vec<OpenElement>,
    name: String,
    attributes: Vec<Attribute>,
) {
    let parent_path = stack
        .last()
        .map_or_else(Vec::new, |element| element.path.clone());
    let children = children_at_path_mut(root, &parent_path);
    let child_index = children.len();
    children.push(Node::Element {
        name: name.clone(),
        attributes,
        children: Vec::new(),
    });
    let mut path = parent_path;
    path.push(child_index);
    stack.push(OpenElement { path, name });
}

fn close_misnested_formatting_element(
    root: &mut Node,
    stack: &mut Vec<OpenElement>,
    name: &str,
) -> bool {
    let Some(position) = stack.iter().rposition(|element| element.name == name) else {
        return false;
    };

    if !is_formatting_element(name) {
        stack.truncate(position);
        return true;
    }

    let reopen: Vec<(String, Vec<Attribute>)> = stack
        .iter()
        .skip(position + 1)
        .filter(|element| is_formatting_element(&element.name))
        .filter_map(|element| match node_at_path(root, &element.path) {
            Node::Element {
                name, attributes, ..
            } => Some((name.clone(), attributes.clone())),
            _ => None,
        })
        .collect();

    let parent_path = stack[position].path[..stack[position].path.len() - 1].to_vec();
    stack.truncate(position);

    let mut current_parent = parent_path;
    for (reopen_name, attributes) in reopen {
        let children = children_at_path_mut(root, &current_parent);
        let child_index = children.len();
        children.push(Node::Element {
            name: reopen_name.clone(),
            attributes,
            children: Vec::new(),
        });
        current_parent.push(child_index);
        stack.push(OpenElement {
            path: current_parent.clone(),
            name: reopen_name,
        });
    }
    true
}

fn is_table_section(name: &str) -> bool {
    matches!(name, "tbody" | "thead" | "tfoot")
}

fn is_table_insertion_context(stack: &[OpenElement]) -> bool {
    matches!(
        stack.last().map(|element| element.name.as_str()),
        Some("table" | "tbody" | "thead" | "tfoot" | "tr")
    )
}

fn is_table_structure_tag(name: &str) -> bool {
    matches!(
        name,
        "caption"
            | "col"
            | "colgroup"
            | "tbody"
            | "td"
            | "tfoot"
            | "th"
            | "thead"
            | "tr"
            | "style"
            | "script"
            | "template"
    )
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
        "tbody" | "thead" | "tfoot" => close_one_of(stack, &["tbody", "thead", "tfoot"]),
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
                let in_table = is_table_insertion_context(&open_elements);
                if in_table
                    && !text.chars().all(char::is_whitespace)
                    && insert_foster_node(&mut root, &mut open_elements, Node::Text(text.clone()))
                        .is_some()
                {
                    continue;
                }
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
                self_closing: _,
            } => {
                apply_implied_end_tags(&mut open_elements, &name);

                if is_table_insertion_context(&open_elements) {
                    let top = open_elements.last().map(|element| element.name.as_str());
                    if name == "tr" && top == Some("table") {
                        append_open_element(
                            &mut root,
                            &mut open_elements,
                            "tbody".into(),
                            Vec::new(),
                        );
                    } else if matches!(name.as_str(), "td" | "th") {
                        if top == Some("table") {
                            append_open_element(
                                &mut root,
                                &mut open_elements,
                                "tbody".into(),
                                Vec::new(),
                            );
                        }
                        if open_elements
                            .last()
                            .is_some_and(|element| is_table_section(&element.name))
                        {
                            append_open_element(
                                &mut root,
                                &mut open_elements,
                                "tr".into(),
                                Vec::new(),
                            );
                        }
                    }

                    if !is_table_structure_tag(&name) && name != "table" {
                        if let Some(path) = insert_foster_node(
                            &mut root,
                            &mut open_elements,
                            Node::Element {
                                name: name.clone(),
                                attributes: attributes.clone(),
                                children: Vec::new(),
                            },
                        ) {
                            if !is_void_element(&name) {
                                open_elements.push(OpenElement { path, name });
                            }
                            continue;
                        }
                    }
                }

                // In the HTML namespace the self-closing flag is ignored for
                // ordinary HTML elements; only void elements stay closed here.
                if !is_void_element(&name) {
                    append_open_element(&mut root, &mut open_elements, name, attributes);
                } else {
                    let path = open_elements
                        .last()
                        .map_or_else(Vec::new, |element| element.path.clone());
                    children_at_path_mut(&mut root, &path).push(Node::Element {
                        name,
                        attributes,
                        children: Vec::new(),
                    });
                }
            }
            Token::EndTag(name) => {
                // A </br> end tag is a parse error that is reprocessed as <br>.
                if name == "br" {
                    let path = open_elements
                        .last()
                        .map_or(&[][..], |element| element.path.as_slice());
                    children_at_path_mut(&mut root, path).push(Node::Element {
                        name: "br".into(),
                        attributes: Vec::new(),
                        children: Vec::new(),
                    });
                    continue;
                }
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
                if matches!(
                    name.as_str(),
                    "table" | "tbody" | "thead" | "tfoot" | "tr" | "td" | "th"
                ) {
                    if let Some(position) = open_elements
                        .iter()
                        .rposition(|element| element.name == name)
                    {
                        open_elements.truncate(position);
                    }
                } else {
                    close_misnested_formatting_element(&mut root, &mut open_elements, &name);
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
            Node::Doctype(_) => {}
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
                        if is_head_content(&other)
                            || is_whitespace_text(&other)
                            || matches!(other, Node::Comment(_))
                        {
                            head.append_child(other).expect("head is a container");
                        } else {
                            // Leaving the head implicitly creates the body.
                            mode = DocumentInsertionMode::AfterHead;
                            pending.push_front(other);
                        }
                    }
                    DocumentInsertionMode::AfterHead => {
                        if is_head_content(&other) {
                            head.append_child(other).expect("head is a container");
                        } else {
                            body.append_child(other).expect("body is a container");
                            mode = DocumentInsertionMode::InBody;
                        }
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
    fn decodes_complete_named_entity_table_including_long_and_multi_codepoint_names() {
        assert_eq!(
            NAMED_ENTITY_DATA.lines().count(),
            2231,
            "generated table should contain the complete HTML5 named-reference set"
        );
        assert_eq!(
            decode_character_references(
                "&CounterClockwiseContourIntegral; &NotEqualTilde; &ThickSpace; &AMP;"
            ),
            "∳ ≂̸    &"
        );
        assert_eq!(decode_character_references("&copy"), "©");
        assert_eq!(decode_character_references("&notit;"), "¬it;");
        assert_eq!(
            decode_character_references_in_attribute("&notit; &amp= &copy"),
            "&notit; &amp= ©"
        );
    }

    #[test]
    fn applies_attribute_ambiguous_ampersand_rule_to_unquoted_values() {
        let tree = parse_fragment("<p a=&notit; b=&copy c=&amp= d=one&amp;two>");
        let paragraph = &tree.children()[0];
        assert_eq!(paragraph.attribute("a"), Some("&notit;"));
        assert_eq!(paragraph.attribute("b"), Some("©"));
        assert_eq!(paragraph.attribute("c"), Some("&amp="));
        assert_eq!(paragraph.attribute("d"), Some("one&two"));
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
    fn foster_parents_stray_table_content_and_inserts_tbody() {
        let tree = parse_fragment("<table>stray<tr><td>cell</td></tr></table>");
        assert_eq!(tree.children().len(), 2);
        assert_eq!(tree.children()[0], Node::Text("stray".into()));

        let table = &tree.children()[1];
        assert_eq!(table.name(), Some("table"));
        assert_eq!(table.elements_by_tag_name("tbody").len(), 1);
        assert_eq!(table.elements_by_tag_name("tr").len(), 1);
        assert_eq!(table.elements_by_tag_name("td")[0].text_content(), "cell");
    }

    #[test]
    fn keeps_normal_content_inside_table_cells() {
        let tree = parse_fragment("<table><tr><td>before<div>inside</div>after</td></tr></table>");
        let cell = &tree.elements_by_tag_name("td")[0];
        assert_eq!(cell.text_content(), "beforeinsideafter");
        assert_eq!(cell.elements_by_tag_name("div").len(), 1);
        assert_eq!(tree.children().len(), 1);
    }

    #[test]
    fn repairs_misnested_inline_formatting_elements() {
        let tree = parse_fragment("<b><i>one</b>two</i>");
        assert_eq!(tree.children().len(), 2);
        assert_eq!(tree.to_html(), "<b><i>one</i></b><i>two</i>");
    }

    #[test]
    fn preprocesses_null_and_line_endings() {
        let tree = parse_fragment("one\\0two\\r\\nthree\\rfour");
        assert_eq!(tree.text_content(), "one�two\\nthree\\nfour");
    }

    #[test]
    fn ignores_self_closing_flag_for_non_void_html_elements() {
        let tree = parse_fragment("<div/>content</div><img/>");
        assert_eq!(tree.to_html(), "<div>content</div><img>");
    }

    #[test]
    fn reprocesses_br_end_tag_as_start_tag() {
        let tree = parse_fragment("before</br>after");
        assert_eq!(tree.to_html(), "before<br>after");
    }

    #[test]
    fn treats_unrecognized_markup_declarations_as_bogus_comments() {
        let tree = parse_fragment("<!legacy>text<?processing?>");
        assert_eq!(tree.to_html(), "<!--legacy-->text<!--processing-->");
    }

    #[test]
    fn handles_abruptly_closed_empty_comments() {
        assert_eq!(parse_fragment("<!-->").to_html(), "<!---->");
        assert_eq!(parse_fragment("<!--->").to_html(), "<!---->");
    }

    #[test]
    fn closes_to_the_matching_open_element() {
        let tree = parse_fragment("<a><b><c>x</b>y</a>");
        let dump = tree.dump();
        assert!(dump.contains("<b>"));
        assert!(dump.contains("#text \"y\""));
    }
}
