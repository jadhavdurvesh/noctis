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
