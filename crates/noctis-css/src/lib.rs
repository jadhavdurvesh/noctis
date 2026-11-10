//! Independent CSS tokenizer and declaration-list parser subset.
use std::char;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Whitespace, Ident(String), AtKeyword(String), Hash(String), String(String),
    Number(String), Percentage(String), Dimension { number: String, unit: String },
    Function(String), Colon, Semicolon, Comma, OpenCurly, CloseCurly,
    OpenParen, CloseParen, OpenSquare, CloseSquare, Delim(char),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    pub property: String,
    pub value: Vec<Token>,
    pub important: bool,
}

fn name_start(c: char) -> bool { c.is_ascii_alphabetic() || c == '_' || !c.is_ascii() }
fn name_char(c: char) -> bool { name_start(c) || c.is_ascii_digit() || c == '-' }
fn newline(c: char) -> bool { matches!(c, '\n' | '\r' | '\u{000c}') }

fn valid_escape(chars: &[char], i: usize) -> bool {
    chars.get(i) == Some(&'\\') && chars.get(i + 1).is_some_and(|c| !newline(*c))
}

fn starts_ident(chars: &[char], i: usize) -> bool {
    match chars.get(i).copied() {
        Some(c) if name_start(c) => true,
        Some('-') => matches!(chars.get(i + 1), Some(c) if name_start(*c) || *c == '-')
            || valid_escape(chars, i + 1),
        Some('\\') => valid_escape(chars, i),
        _ => false,
    }
}

fn starts_number(chars: &[char], i: usize) -> bool {
    match (chars.get(i), chars.get(i + 1)) {
        (Some('+' | '-'), Some(next)) => next.is_ascii_digit()
            || (*next == '.' && chars.get(i + 2).is_some_and(|c| c.is_ascii_digit())),
        (Some('.'), Some(next)) => next.is_ascii_digit(),
        (Some(c), _) => c.is_ascii_digit(),
        _ => false,
    }
}

fn consume_escape(chars: &[char], i: &mut usize, out: &mut String) {
    *i += 1;
    if *i >= chars.len() { out.push('\u{fffd}'); return; }
    if chars[*i].is_ascii_hexdigit() {
        let start = *i;
        let mut count = 0;
        while *i < chars.len() && count < 6 && chars[*i].is_ascii_hexdigit() {
            *i += 1; count += 1;
        }
        let digits: String = chars[start..*i].iter().collect();
        let value = u32::from_str_radix(&digits, 16).unwrap_or(0xfffd);
        out.push(char::from_u32(value).filter(|c| *c != '\0').unwrap_or('\u{fffd}'));
        if *i < chars.len() && chars[*i].is_whitespace() { *i += 1; }
    } else {
        out.push(chars[*i]); *i += 1;
    }
}

fn consume_name(chars: &[char], i: &mut usize) -> String {
    let mut out = String::new();
    while *i < chars.len() {
        if name_char(chars[*i]) { out.push(chars[*i]); *i += 1; }
        else if valid_escape(chars, *i) { consume_escape(chars, i, &mut out); }
        else { break; }
    }
    out
}

fn consume_string(chars: &[char], i: &mut usize, quote: char) -> String {
    *i += 1;
    let mut out = String::new();
    while *i < chars.len() {
        let c = chars[*i];
        if c == quote { *i += 1; break; }
        if c == '\\' && *i + 1 < chars.len() && !newline(chars[*i + 1]) {
            consume_escape(chars, i, &mut out);
        } else { out.push(c); *i += 1; }
    }
    out
}

fn consume_number(chars: &[char], i: &mut usize) -> String {
    let start = *i;
    if matches!(chars.get(*i), Some('+' | '-')) { *i += 1; }
    while *i < chars.len() && chars[*i].is_ascii_digit() { *i += 1; }
    if chars.get(*i) == Some(&'.') && chars.get(*i + 1).is_some_and(|c| c.is_ascii_digit()) {
        *i += 1;
        while *i < chars.len() && chars[*i].is_ascii_digit() { *i += 1; }
    }
    if matches!(chars.get(*i), Some('e' | 'E')) {
        let mut e = *i + 1;
        if matches!(chars.get(e), Some('+' | '-')) { e += 1; }
        if chars.get(e).is_some_and(|c| c.is_ascii_digit()) {
            *i = e + 1;
            while *i < chars.len() && chars[*i].is_ascii_digit() { *i += 1; }
        }
    }
    chars[start..*i].iter().collect()
}

/// Tokenize the supported CSS Syntax subset. Comments are omitted.
pub fn tokenize(input: &str) -> Vec<Token> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            while i < chars.len() && chars[i].is_whitespace() { i += 1; }
            tokens.push(Token::Whitespace); continue;
        }
        if c == '/' && chars.get(i + 1) == Some(&'*') {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') { i += 1; }
            i = (i + 2).min(chars.len()); continue;
        }
        if c == '"' || c == '\'' {
            tokens.push(Token::String(consume_string(&chars, &mut i, c))); continue;
        }
        if c == '@' && starts_ident(&chars, i + 1) {
            i += 1; tokens.push(Token::AtKeyword(consume_name(&chars, &mut i))); continue;
        }
        if c == '#' && (starts_ident(&chars, i + 1) || chars.get(i + 1).is_some_and(|c| c.is_ascii_digit())) {
            i += 1; tokens.push(Token::Hash(consume_name(&chars, &mut i))); continue;
        }
        if starts_number(&chars, i) {
            let number = consume_number(&chars, &mut i);
            if chars.get(i) == Some(&'%') { i += 1; tokens.push(Token::Percentage(number)); }
            else if starts_ident(&chars, i) {
                let unit = consume_name(&chars, &mut i);
                tokens.push(Token::Dimension { number, unit });
            } else { tokens.push(Token::Number(number)); }
            continue;
        }
        if starts_ident(&chars, i) {
            let name = consume_name(&chars, &mut i);
            if chars.get(i) == Some(&'(') { i += 1; tokens.push(Token::Function(name)); }
            else { tokens.push(Token::Ident(name)); }
            continue;
        }
        i += 1;
        tokens.push(match c {
            ':' => Token::Colon, ';' => Token::Semicolon, ',' => Token::Comma,
            '{' => Token::OpenCurly, '}' => Token::CloseCurly,
            '(' => Token::OpenParen, ')' => Token::CloseParen,
            '[' => Token::OpenSquare, ']' => Token::CloseSquare,
            _ => Token::Delim(c),
        });
    }
    tokens
}

fn trim_ws(tokens: &mut Vec<Token>) {
    while matches!(tokens.first(), Some(Token::Whitespace)) { tokens.remove(0); }
    while matches!(tokens.last(), Some(Token::Whitespace)) { tokens.pop(); }
}

fn remove_important(tokens: &mut Vec<Token>) -> bool {
    let mut end = tokens.len();
    while end > 0 && matches!(tokens[end - 1], Token::Whitespace) { end -= 1; }
    if end == 0 || !matches!(&tokens[end - 1], Token::Ident(name) if name.eq_ignore_ascii_case("important")) {
        return false;
    }
    let mut bang = end - 1;
    while bang > 0 && matches!(tokens[bang - 1], Token::Whitespace) { bang -= 1; }
    if bang == 0 || !matches!(tokens[bang - 1], Token::Delim('!')) { return false; }
    tokens.truncate(bang - 1);
    trim_ws(tokens);
    true
}

fn parse_one(tokens: &[Token]) -> Option<Declaration> {
    let colon = tokens.iter().position(|t| matches!(t, Token::Colon))?;
    let [Token::Ident(property)] = &tokens[..colon] else { return None; };
    let mut value = tokens[colon + 1..].to_vec();
    trim_ws(&mut value);
    if value.is_empty() { return None; }
    let important = remove_important(&mut value);
    if value.is_empty() { return None; }
    let property = if property.starts_with("--") { property.clone() } else { property.to_ascii_lowercase() };
    Some(Declaration { property, value, important })
}

/// Parse declarations at top-level semicolons, ignoring semicolons nested in
/// functions, parentheses, brackets and blocks. Invalid declarations are skipped.
pub fn parse_declaration_list(input: &str) -> Vec<Declaration> {
    let tokens = tokenize(input);
    let mut result = Vec::new();
    let mut start = 0;
    let (mut parens, mut brackets, mut braces) = (0usize, 0usize, 0usize);
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::Function(_) | Token::OpenParen => parens += 1,
            Token::CloseParen => parens = parens.saturating_sub(1),
            Token::OpenSquare => brackets += 1,
            Token::CloseSquare => brackets = brackets.saturating_sub(1),
            Token::OpenCurly => braces += 1,
            Token::CloseCurly => braces = braces.saturating_sub(1),
            Token::Semicolon if parens == 0 && brackets == 0 && braces == 0 => {
                if let Some(declaration) = parse_one(&tokens[start..i]) { result.push(declaration); }
                start = i + 1;
            }
            _ => {}
        }
    }
    if let Some(declaration) = parse_one(&tokens[start..]) { result.push(declaration); }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_dimensions_percentages_and_identifiers() {
        assert_eq!(tokenize("width: 12px; opacity: 50%"), vec![
            Token::Ident("width".into()), Token::Colon, Token::Whitespace,
            Token::Dimension { number: "12".into(), unit: "px".into() },
            Token::Semicolon, Token::Whitespace, Token::Ident("opacity".into()),
            Token::Colon, Token::Whitespace, Token::Percentage("50".into()),
        ]);
    }

    #[test]
    fn removes_comments_and_decodes_escaped_names() {
        assert_eq!(tokenize("co/*x*/lor: r\\65 d"), vec![
            Token::Ident("co".into()), Token::Ident("lor".into()), Token::Colon,
            Token::Whitespace, Token::Ident("red".into()),
        ]);
    }

    #[test]
    fn parses_strings_and_functions() {
        assert_eq!(tokenize("content: \"hello\"; image: url('a.png')"), vec![
            Token::Ident("content".into()), Token::Colon, Token::Whitespace,
            Token::String("hello".into()), Token::Semicolon, Token::Whitespace,
            Token::Ident("image".into()), Token::Colon, Token::Whitespace,
            Token::Function("url".into()), Token::String("a.png".into()), Token::CloseParen,
        ]);
    }

    #[test]
    fn parses_declarations_and_important() {
        let declarations = parse_declaration_list("COLOR: red; margin: 1px 2px ! IMPORTANT; --Brand: NOCTIS");
        assert_eq!(declarations.len(), 3);
        assert_eq!(declarations[0].property, "color");
        assert!(declarations[1].important);
        assert_eq!(declarations[2].property, "--Brand");
    }

    #[test]
    fn does_not_split_semicolons_inside_functions_or_blocks() {
        let declarations = parse_declaration_list("background: image-set(url('a;b.png') 1x); --x: { a: b; c: d }; color: blue");
        assert_eq!(declarations.len(), 3);
        assert_eq!(declarations[0].property, "background");
        assert_eq!(declarations[1].property, "--x");
        assert_eq!(declarations[2].property, "color");
    }

    #[test]
    fn skips_malformed_declarations_and_keeps_valid_neighbors() {
        let declarations = parse_declaration_list("color red; : invalid; color: green; width: 4px");
        assert_eq!(declarations.len(), 2);
        assert_eq!(declarations[0].property, "color");
        assert_eq!(declarations[1].property, "width");
    }
}

use noctis_html::{DomDocument, DomNodeId, DomNodeKind};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Specificity { pub ids: u16, pub classes: u16, pub types: u16 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeOperator { Equals, Includes, DashMatch, Prefix, Suffix, Substring }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeSelector { pub name: String, pub operator: Option<AttributeOperator>, pub value: Option<String> }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleSelector {
    pub tag_name: Option<String>, pub ids: Vec<String>,
    pub classes: Vec<String>, pub attributes: Vec<AttributeSelector>,
}

impl SimpleSelector {
    pub fn specificity(&self) -> Specificity {
        Specificity {
            ids: self.ids.len().min(u16::MAX as usize) as u16,
            classes: (self.classes.len() + self.attributes.len()).min(u16::MAX as usize) as u16,
            types: u16::from(self.tag_name.is_some()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleRule { pub selectors: Vec<SimpleSelector>, pub declarations: Vec<Declaration> }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedProperty { pub property: String, pub value: Vec<Token>, pub important: bool }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CascadePriority { important: bool, inline: bool, specificity: Specificity, order: usize }

#[derive(Debug, Clone)]
struct CascadeCandidate { priority: CascadePriority, value: Vec<Token>, important: bool }

/// Parse a supported compound selector: type, universal, IDs, classes and
/// attribute selectors. Combinators and pseudo-classes are intentionally rejected.
pub fn parse_simple_selector(input: &str) -> Option<SimpleSelector> {
    let chars: Vec<char> = input.trim().chars().collect();
    if chars.is_empty() { return None; }
    let mut selector = SimpleSelector { tag_name: None, ids: Vec::new(), classes: Vec::new(), attributes: Vec::new() };
    let mut i = 0;
    if chars[i] == '*' { i += 1; }
    else if chars[i].is_ascii_alphabetic() || chars[i] == '_' || chars[i] == '-' {
        let start = i;
        while i < chars.len() && (chars[i].is_ascii_alphanumeric() || matches!(chars[i], '-' | '_')) { i += 1; }
        selector.tag_name = Some(chars[start..i].iter().collect::<String>().to_ascii_lowercase());
    }
    while i < chars.len() {
        match chars[i] {
            '#' | '.' => {
                let marker = chars[i]; i += 1; let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || matches!(chars[i], '-' | '_')) { i += 1; }
                if start == i { return None; }
                let value: String = chars[start..i].iter().collect();
                if marker == '#' { selector.ids.push(value); } else { selector.classes.push(value); }
            }
            '[' => {
                i += 1;
                while i < chars.len() && chars[i].is_whitespace() { i += 1; }
                let start = i;
                while i < chars.len() && !chars[i].is_whitespace() && !matches!(chars[i], '=' | ']' | '~' | '|' | '^' | '$' | '*') { i += 1; }
                if start == i { return None; }
                let name = chars[start..i].iter().collect::<String>().to_ascii_lowercase();
                while i < chars.len() && chars[i].is_whitespace() { i += 1; }
                let mut operator = None;
                let mut value = None;
                if i < chars.len() && chars[i] != ']' {
                    operator = Some(match chars[i] {
                        '=' => { i += 1; AttributeOperator::Equals }
                        marker @ ('~' | '|' | '^' | '$' | '*') => {
                            if chars.get(i + 1) != Some(&'=') { return None; }
                            i += 2;
                            match marker {
                                '~' => AttributeOperator::Includes,
                                '|' => AttributeOperator::DashMatch,
                                '^' => AttributeOperator::Prefix,
                                '$' => AttributeOperator::Suffix,
                                '*' => AttributeOperator::Substring,
                                _ => unreachable!(),
                            }
                        }
                        _ => return None,
                    });
                    while i < chars.len() && chars[i].is_whitespace() { i += 1; }
                    if i >= chars.len() || chars[i] == ']' { return None; }
                    let quote = if matches!(chars[i], '"' | '\'') { let q = chars[i]; i += 1; Some(q) } else { None };
                    let value_start = i;
                    if let Some(q) = quote {
                        while i < chars.len() && chars[i] != q { i += 1; }
                        if i >= chars.len() { return None; }
                        value = Some(chars[value_start..i].iter().collect()); i += 1;
                    } else {
                        while i < chars.len() && !chars[i].is_whitespace() && chars[i] != ']' { i += 1; }
                        value = Some(chars[value_start..i].iter().collect());
                    }
                    while i < chars.len() && chars[i].is_whitespace() { i += 1; }
                }
                if chars.get(i) != Some(&']') { return None; }
                i += 1;
                selector.attributes.push(AttributeSelector { name, operator, value });
            }
            _ => return None,
        }
    }
    Some(selector)
}

pub fn parse_selector_list(input: &str) -> Vec<SimpleSelector> {
    let chars: Vec<char> = input.chars().collect();
    let mut selectors = Vec::new();
    let mut start = 0;
    let mut brackets = 0usize;
    let mut quote = None;
    for (i, c) in chars.iter().enumerate() {
        if let Some(q) = quote { if *c == q { quote = None; } continue; }
        match c {
            '\'' | '"' if brackets > 0 => quote = Some(*c),
            '[' => brackets += 1,
            ']' => brackets = brackets.saturating_sub(1),
            ',' if brackets == 0 => {
                if let Some(selector) = parse_simple_selector(&chars[start..i].iter().collect::<String>()) { selectors.push(selector); }
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < chars.len() {
        if let Some(selector) = parse_simple_selector(&chars[start..].iter().collect::<String>()) { selectors.push(selector); }
    }
    selectors
}

fn attribute_matches(kind: &DomNodeKind, selector: &AttributeSelector) -> bool {
    if !kind.has_attribute(&selector.name) { return false; }
    let Some(operator) = selector.operator else { return true; };
    let Some(actual) = kind.attribute(&selector.name) else { return false; };
    let expected = selector.value.as_deref().unwrap_or_default();
    match operator {
        AttributeOperator::Equals => actual == expected,
        AttributeOperator::Includes => actual.split_ascii_whitespace().any(|part| part == expected),
        AttributeOperator::DashMatch => actual == expected || actual.strip_prefix(expected).is_some_and(|rest| rest.starts_with('-')),
        AttributeOperator::Prefix => !expected.is_empty() && actual.starts_with(expected),
        AttributeOperator::Suffix => !expected.is_empty() && actual.ends_with(expected),
        AttributeOperator::Substring => !expected.is_empty() && actual.contains(expected),
    }
}

pub fn matches_selector(document: &DomDocument, node_id: DomNodeId, selector: &SimpleSelector) -> bool {
    let Ok(DomNodeKind::Element { name, attributes }) = document.kind(node_id) else { return false; };
    let kind = DomNodeKind::Element { name: name.clone(), attributes: attributes.clone() };
    if selector.tag_name.as_deref().is_some_and(|tag| !name.eq_ignore_ascii_case(tag)) { return false; }
    if selector.ids.iter().any(|id| kind.attribute("id") != Some(id.as_str())) { return false; }
    let classes: Vec<&str> = kind.attribute("class").unwrap_or_default().split_ascii_whitespace().collect();
    if selector.classes.iter().any(|class| !classes.contains(&class.as_str())) { return false; }
    selector.attributes.iter().all(|attribute| attribute_matches(&kind, attribute))
}

fn find_rule_end(chars: &[char], mut i: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut quote = None;
    while i < chars.len() {
        let c = chars[i];
        if let Some(q) = quote {
            if c == '\\' { i = (i + 2).min(chars.len()); continue; }
            if c == q { quote = None; }
            i += 1; continue;
        }
        if c == '"' || c == '\'' { quote = Some(c); i += 1; }
        else if c == '/' && chars.get(i + 1) == Some(&'*') {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') { i += 1; }
            i = (i + 2).min(chars.len());
        } else if c == '{' { depth += 1; i += 1; }
        else if c == '}' { depth -= 1; if depth == 0 { return Some(i); } i += 1; }
        else { i += 1; }
    }
    None
}

/// Parse ordinary rules and collect rules in @media/@supports/@layer blocks.
/// Conditions are parsed as grouping only; they are not evaluated.
pub fn parse_stylesheet(input: &str) -> Vec<StyleRule> {
    fn level(input: &str, output: &mut Vec<StyleRule>, nesting: usize) {
        if nesting > 16 { return; }
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            while i < chars.len() && chars[i].is_whitespace() { i += 1; }
            if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') { i += 1; }
                i = (i + 2).min(chars.len()); continue;
            }
            let prelude_start = i;
            let mut quote = None;
            let mut brackets = 0usize;
            let mut parens = 0usize;
            while i < chars.len() {
                let c = chars[i];
                if let Some(q) = quote {
                    if c == '\\' { i = (i + 2).min(chars.len()); continue; }
                    if c == q { quote = None; }
                    i += 1; continue;
                }
                match c {
                    '\'' | '"' => quote = Some(c),
                    '[' => brackets += 1,
                    ']' => brackets = brackets.saturating_sub(1),
                    '(' => parens += 1,
                    ')' => parens = parens.saturating_sub(1),
                    '{' if brackets == 0 && parens == 0 => break,
                    ';' | '}' if brackets == 0 && parens == 0 => break,
                    _ => {}
                }
                i += 1;
            }
            if i >= chars.len() || chars[i] != '{' { i = (i + 1).min(chars.len()); continue; }
            let prelude = chars[prelude_start..i].iter().collect::<String>();
            let body_start = i + 1;
            let Some(body_end) = find_rule_end(&chars, body_start) else { break; };
            let body = chars[body_start..body_end].iter().collect::<String>();
            let trimmed = prelude.trim();
            if trimmed.starts_with("@media") || trimmed.starts_with("@supports") || trimmed.starts_with("@layer") {
                level(&body, output, nesting + 1);
            } else if !trimmed.starts_with('@') {
                let selectors = parse_selector_list(trimmed);
                let declarations = parse_declaration_list(&body);
                if !selectors.is_empty() && !declarations.is_empty() { output.push(StyleRule { selectors, declarations }); }
            }
            i = body_end + 1;
        }
    }
    let mut rules = Vec::new();
    level(input, &mut rules, 0);
    rules
}

/// Compute winning declarations for one node in this scoped cascade.
/// Inheritance, pseudo-classes, combinators and media condition evaluation
/// remain future work.
pub fn compute_style(document: &DomDocument, node_id: DomNodeId, rules: &[StyleRule]) -> Vec<ComputedProperty> {
    let mut winners: BTreeMap<String, CascadeCandidate> = BTreeMap::new();
    let mut order = 0usize;
    for rule in rules {
        let specificity = rule.selectors.iter()
            .filter(|selector| matches_selector(document, node_id, selector))
            .map(SimpleSelector::specificity).max();
        let Some(specificity) = specificity else { continue; };
        for declaration in &rule.declarations {
            let priority = CascadePriority { important: declaration.important, inline: false, specificity, order };
            let candidate = CascadeCandidate { priority, value: declaration.value.clone(), important: declaration.important };
            if winners.get(&declaration.property).map_or(true, |current| priority >= current.priority) {
                winners.insert(declaration.property.clone(), candidate);
            }
            order += 1;
        }
    }
    if let Ok(DomNodeKind::Element { attributes, .. }) = document.kind(node_id) {
        if let Some(style) = attributes.iter()
            .find(|attribute| attribute.name.eq_ignore_ascii_case("style"))
            .and_then(|attribute| attribute.value.as_deref())
        {
            for declaration in parse_declaration_list(style) {
                let priority = CascadePriority { important: declaration.important, inline: true, specificity: Specificity::default(), order };
                let candidate = CascadeCandidate { priority, value: declaration.value, important: declaration.important };
                if winners.get(&declaration.property).map_or(true, |current| priority >= current.priority) {
                    winners.insert(declaration.property, candidate);
                }
                order += 1;
            }
        }
    }
    winners.into_iter().map(|(property, candidate)| ComputedProperty {
        property, value: candidate.value, important: candidate.important,
    }).collect()
}

#[cfg(test)]
mod selector_tests {
    use super::*;
    use noctis_html::{parse_fragment, DomDocument};

    #[test]
    fn parses_compound_and_attribute_selectors() {
        let selector = parse_simple_selector("article.card#main[data-kind='news'][hidden]").unwrap();
        assert_eq!(selector.tag_name.as_deref(), Some("article"));
        assert_eq!(selector.ids, vec!["main"]);
        assert_eq!(selector.classes, vec!["card"]);
        assert_eq!(selector.attributes.len(), 2);
        assert_eq!(selector.specificity(), Specificity { ids: 1, classes: 3, types: 1 });
    }

    #[test]
    fn matches_types_ids_classes_and_attribute_operators() {
        let tree = parse_fragment("<article id='main' class='card featured' data-kind='news item' lang='en-US'></article>");
        let document = DomDocument::from_tree(&tree).unwrap();
        let node = document.elements_by_tag_name("article")[0];
        for selector in ["article.card#main", "[data-kind~='news']", "[lang|='en']", "[data-kind^='news']", "[lang$='US']", "[data-kind*='ws i']"] {
            let parsed = parse_simple_selector(selector).unwrap();
            assert!(matches_selector(&document, node, &parsed), "{selector}");
        }
        assert!(!matches_selector(&document, node, &parse_simple_selector("#other").unwrap()));
    }

    #[test]
    fn parses_stylesheet_rules_and_nested_conditional_groups() {
        let rules = parse_stylesheet("p, .note { color: red; } @media screen { #main { width: 10px; } }");
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].selectors.len(), 2);
        assert_eq!(rules[0].declarations.len(), 1);
        assert_eq!(rules[1].selectors[0].ids, vec!["main"]);
    }

    #[test]
    fn computes_specificity_inline_and_important_cascade() {
        let tree = parse_fragment("<p id='hero' class='note' style='color: black'></p>");
        let document = DomDocument::from_tree(&tree).unwrap();
        let node = document.elements_by_tag_name("p")[0];
        let rules = parse_stylesheet("p { color: blue; } .note { color: green !important; } #hero { color: red; }");
        let style = compute_style(&document, node, &rules);
        assert_eq!(style.len(), 1);
        assert_eq!(style[0].property, "color");
        assert!(style[0].important);
        assert_eq!(style[0].value, vec![Token::Ident("green".into())]);
    }
}
