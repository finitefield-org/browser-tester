use super::html::{can_start_regex_literal, unescape_string};
use super::*;

mod parser_expr;
mod parser_stmt;

use parser_stmt::is_ident_char;

pub(super) fn parse_expr(src: &str) -> Result<Expr> {
    parser_expr::parse_expr(src)
}

pub(super) fn parse_function_expr(src: &str) -> Result<Option<Expr>> {
    parser_stmt::parse_function_expr(src)
}

pub(super) fn parse_block_statements(body: &str) -> Result<Vec<Stmt>> {
    parser_stmt::parse_block_statements(body)
}

#[cfg(test)]
pub(super) fn parse_for_each_callback(src: &str) -> Result<(String, Option<String>, Vec<Stmt>)> {
    parser_stmt::parse_for_each_callback(src)
}

pub(super) fn resolve_insert_adjacent_position(src: &str) -> Result<InsertAdjacentPosition> {
    parser_stmt::resolve_insert_adjacent_position(src)
}

pub(super) fn is_ident(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
        return false;
    }

    chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

pub(super) fn identifier_allows_regex_start(
    ident: &[u8],
    previous_significant: Option<u8>,
) -> bool {
    if matches!(previous_significant, Some(b'.')) {
        return false;
    }
    matches!(
        ident,
        b"return"
            | b"throw"
            | b"case"
            | b"delete"
            | b"typeof"
            | b"void"
            | b"yield"
            | b"await"
            | b"in"
            | b"of"
            | b"instanceof"
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum JsLexMode {
    Normal,
    Single,
    Double,
    Backtick,
    TemplateExpr { brace_depth: usize },
    Regex { in_class: bool },
    LineComment,
    BlockComment,
}

#[derive(Debug)]
pub(super) struct JsLexScanner {
    mode: JsLexMode,
    mode_stack: Vec<JsLexMode>,
    paren: usize,
    bracket: usize,
    brace: usize,
    previous_significant: Option<u8>,
    previous_identifier_allows_regex: bool,
}

impl JsLexScanner {
    pub(super) fn new() -> Self {
        Self {
            mode: JsLexMode::Normal,
            mode_stack: Vec::new(),
            paren: 0,
            bracket: 0,
            brace: 0,
            previous_significant: None,
            previous_identifier_allows_regex: false,
        }
    }

    pub(super) fn in_normal(&self) -> bool {
        matches!(self.mode, JsLexMode::Normal)
    }

    pub(super) fn is_top_level(&self) -> bool {
        self.in_normal() && self.paren == 0 && self.bracket == 0 && self.brace == 0
    }

    pub(super) fn consume_significant_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.note_significant_byte(b);
        }
    }

    pub(super) fn slash_starts_comment_or_regex(&self, bytes: &[u8], i: usize) -> bool {
        if !self.in_normal() || bytes.get(i).copied() != Some(b'/') {
            return false;
        }
        if i + 1 < bytes.len() && (bytes[i + 1] == b'/' || bytes[i + 1] == b'*') {
            return true;
        }
        can_start_regex_literal(self.previous_significant) || self.previous_identifier_allows_regex
    }

    fn note_significant_byte(&mut self, b: u8) {
        match b {
            b'(' => self.paren += 1,
            b')' => self.paren = self.paren.saturating_sub(1),
            b'[' => self.bracket += 1,
            b']' => self.bracket = self.bracket.saturating_sub(1),
            b'{' => self.brace += 1,
            b'}' => self.brace = self.brace.saturating_sub(1),
            _ => {}
        }
        self.previous_significant = Some(b);
        self.previous_identifier_allows_regex = false;
    }

    fn push_mode(&mut self, next: JsLexMode) {
        self.mode_stack.push(self.mode);
        self.mode = next;
    }

    fn pop_mode(&mut self) {
        self.mode = self.mode_stack.pop().unwrap_or(JsLexMode::Normal);
    }

    pub(super) fn advance(&mut self, bytes: &[u8], i: usize) -> usize {
        let b = bytes[i];
        match self.mode {
            JsLexMode::Normal => {
                if b.is_ascii_whitespace() {
                    return i + 1;
                }
                if b == b'_' || b == b'$' || b.is_ascii_alphabetic() {
                    let start = i;
                    let mut end = i + 1;
                    while end < bytes.len() && is_ident_char(bytes[end]) {
                        end += 1;
                    }
                    let prev = self.previous_significant;
                    self.previous_significant = Some(bytes[end - 1]);
                    self.previous_identifier_allows_regex =
                        identifier_allows_regex_start(&bytes[start..end], prev);
                    return end;
                }
                match b {
                    b'\'' => {
                        self.push_mode(JsLexMode::Single);
                        self.previous_identifier_allows_regex = false;
                        i + 1
                    }
                    b'"' => {
                        self.push_mode(JsLexMode::Double);
                        self.previous_identifier_allows_regex = false;
                        i + 1
                    }
                    b'`' => {
                        self.push_mode(JsLexMode::Backtick);
                        self.previous_identifier_allows_regex = false;
                        i + 1
                    }
                    b'/' => {
                        if i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                            self.push_mode(JsLexMode::LineComment);
                            i + 2
                        } else if i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                            self.push_mode(JsLexMode::BlockComment);
                            i + 2
                        } else if can_start_regex_literal(self.previous_significant)
                            || self.previous_identifier_allows_regex
                        {
                            self.push_mode(JsLexMode::Regex { in_class: false });
                            self.previous_identifier_allows_regex = false;
                            i + 1
                        } else {
                            self.note_significant_byte(b'/');
                            i + 1
                        }
                    }
                    _ => {
                        self.note_significant_byte(b);
                        i + 1
                    }
                }
            }
            JsLexMode::Single => {
                if b == b'\\' {
                    (i + 2).min(bytes.len())
                } else {
                    if b == b'\'' {
                        self.pop_mode();
                        self.previous_significant = Some(b'\'');
                        self.previous_identifier_allows_regex = false;
                    }
                    i + 1
                }
            }
            JsLexMode::Double => {
                if b == b'\\' {
                    (i + 2).min(bytes.len())
                } else {
                    if b == b'"' {
                        self.pop_mode();
                        self.previous_significant = Some(b'"');
                        self.previous_identifier_allows_regex = false;
                    }
                    i + 1
                }
            }
            JsLexMode::Backtick => {
                if b == b'\\' {
                    (i + 2).min(bytes.len())
                } else if b == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                    self.push_mode(JsLexMode::TemplateExpr { brace_depth: 1 });
                    i + 2
                } else {
                    if b == b'`' {
                        self.pop_mode();
                        self.previous_significant = Some(b'`');
                        self.previous_identifier_allows_regex = false;
                    }
                    i + 1
                }
            }
            JsLexMode::TemplateExpr { mut brace_depth } => {
                if b.is_ascii_whitespace() {
                    return i + 1;
                }
                if b == b'_' || b == b'$' || b.is_ascii_alphabetic() {
                    let start = i;
                    let mut end = i + 1;
                    while end < bytes.len() && is_ident_char(bytes[end]) {
                        end += 1;
                    }
                    let prev = self.previous_significant;
                    self.previous_significant = Some(bytes[end - 1]);
                    self.previous_identifier_allows_regex =
                        identifier_allows_regex_start(&bytes[start..end], prev);
                    return end;
                }
                match b {
                    b'\'' => {
                        self.push_mode(JsLexMode::Single);
                        self.previous_identifier_allows_regex = false;
                        i + 1
                    }
                    b'"' => {
                        self.push_mode(JsLexMode::Double);
                        self.previous_identifier_allows_regex = false;
                        i + 1
                    }
                    b'`' => {
                        self.push_mode(JsLexMode::Backtick);
                        self.previous_identifier_allows_regex = false;
                        i + 1
                    }
                    b'/' => {
                        if i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                            self.push_mode(JsLexMode::LineComment);
                            i + 2
                        } else if i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                            self.push_mode(JsLexMode::BlockComment);
                            i + 2
                        } else if can_start_regex_literal(self.previous_significant)
                            || self.previous_identifier_allows_regex
                        {
                            self.push_mode(JsLexMode::Regex { in_class: false });
                            self.previous_identifier_allows_regex = false;
                            i + 1
                        } else {
                            self.note_significant_byte(b'/');
                            i + 1
                        }
                    }
                    b'{' => {
                        brace_depth += 1;
                        self.note_significant_byte(b'{');
                        self.mode = JsLexMode::TemplateExpr { brace_depth };
                        i + 1
                    }
                    b'}' => {
                        if brace_depth == 1 {
                            self.pop_mode();
                            self.previous_significant = Some(b'}');
                            self.previous_identifier_allows_regex = false;
                        } else {
                            brace_depth -= 1;
                            self.note_significant_byte(b'}');
                            self.mode = JsLexMode::TemplateExpr { brace_depth };
                        }
                        i + 1
                    }
                    _ => {
                        self.note_significant_byte(b);
                        i + 1
                    }
                }
            }
            JsLexMode::LineComment => {
                if b == b'\n' || b == b'\r' {
                    self.pop_mode();
                }
                i + 1
            }
            JsLexMode::BlockComment => {
                if b == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    self.pop_mode();
                    i + 2
                } else {
                    i + 1
                }
            }
            JsLexMode::Regex { mut in_class } => {
                if b == b'\\' {
                    return (i + 2).min(bytes.len());
                }
                if b == b'[' {
                    in_class = true;
                    self.mode = JsLexMode::Regex { in_class };
                    return i + 1;
                }
                if b == b']' && in_class {
                    in_class = false;
                    self.mode = JsLexMode::Regex { in_class };
                    return i + 1;
                }
                if b == b'/' && !in_class {
                    self.pop_mode();
                    self.previous_significant = Some(b'/');
                    self.previous_identifier_allows_regex = false;
                    return i + 1;
                }
                self.mode = JsLexMode::Regex { in_class };
                i + 1
            }
        }
    }
}

#[derive(Debug)]
struct Cursor<'a> {
    src: &'a str,
    i: usize,
}

impl<'a> Cursor<'a> {
    pub(super) fn new(src: &'a str) -> Self {
        Self { src, i: 0 }
    }

    pub(super) fn eof(&self) -> bool {
        self.i >= self.src.len()
    }

    pub(super) fn pos(&self) -> usize {
        self.i
    }

    pub(super) fn set_pos(&mut self, pos: usize) {
        self.i = pos;
    }

    pub(super) fn bytes(&self) -> &'a [u8] {
        self.src.as_bytes()
    }

    pub(super) fn peek(&self) -> Option<u8> {
        self.bytes().get(self.i).copied()
    }

    pub(super) fn consume_byte(&mut self, b: u8) -> bool {
        if self.peek() == Some(b) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    pub(super) fn expect_byte(&mut self, b: u8) -> Result<()> {
        if self.consume_byte(b) {
            Ok(())
        } else {
            Err(Error::ScriptParse(format!(
                "expected '{}' at {}",
                b as char, self.i
            )))
        }
    }

    pub(super) fn consume_ascii(&mut self, token: &str) -> bool {
        let bytes = self.bytes();
        if self.i + token.len() > bytes.len() {
            return false;
        }
        let got = &bytes[self.i..self.i + token.len()];
        if got == token.as_bytes() {
            self.i += token.len();
            true
        } else {
            false
        }
    }

    pub(super) fn expect_ascii(&mut self, token: &str) -> Result<()> {
        if self.consume_ascii(token) {
            Ok(())
        } else {
            Err(Error::ScriptParse(format!(
                "expected '{}' at {}",
                token, self.i
            )))
        }
    }

    pub(super) fn skip_ws(&mut self) {
        self.skip_ws_and_comments()
    }

    pub(super) fn skip_ws_and_comments(&mut self) {
        loop {
            self.skip_plain_ws();
            if self.consume_ascii("//") {
                while let Some(b) = self.peek() {
                    self.i += 1;
                    if b == b'\n' {
                        break;
                    }
                }
                continue;
            }
            if self.consume_ascii("/*") {
                while !self.eof() {
                    if self.consume_ascii("*/") {
                        break;
                    }
                    self.i += 1;
                }
                continue;
            }
            break;
        }
    }

    pub(super) fn skip_plain_ws(&mut self) {
        while let Some(b) = self.peek() {
            if b.is_ascii_whitespace() {
                self.i += 1;
            } else {
                break;
            }
        }
    }

    pub(super) fn parse_identifier(&mut self) -> Option<String> {
        let bytes = self.bytes();
        let start = self.i;
        let first = *bytes.get(self.i)?;
        if !(first == b'_' || first == b'$' || first.is_ascii_alphabetic()) {
            return None;
        }
        self.i += 1;
        while let Some(b) = bytes.get(self.i).copied() {
            if b == b'_' || b == b'$' || b.is_ascii_alphanumeric() {
                self.i += 1;
            } else {
                break;
            }
        }
        self.src.get(start..self.i).map(|s| s.to_string())
    }

    pub(super) fn parse_string_literal(&mut self) -> Result<String> {
        let quote = self
            .peek()
            .ok_or_else(|| Error::ScriptParse("expected string literal".into()))?;
        if quote != b'\'' && quote != b'"' {
            return Err(Error::ScriptParse(format!(
                "expected string literal at {}",
                self.i
            )));
        }

        self.i += 1;
        let start = self.i;

        let bytes = self.bytes();
        while self.i < bytes.len() {
            let b = bytes[self.i];
            if b == b'\\' {
                self.i += 2;
                continue;
            }
            if b == quote {
                let raw = self
                    .src
                    .get(start..self.i)
                    .ok_or_else(|| Error::ScriptParse("invalid string literal".into()))?;
                self.i += 1;
                return Ok(unescape_string(raw));
            }
            self.i += 1;
        }

        Err(Error::ScriptParse("unclosed string literal".into()))
    }

    pub(super) fn read_until_byte(&mut self, b: u8) -> Result<String> {
        let start = self.i;
        while let Some(current) = self.peek() {
            if current == b {
                return self
                    .src
                    .get(start..self.i)
                    .map(|s| s.to_string())
                    .ok_or_else(|| Error::ScriptParse("invalid substring".into()));
            }
            self.i += 1;
        }
        Err(Error::ScriptParse(format!(
            "expected '{}' before EOF",
            b as char
        )))
    }

    pub(super) fn read_balanced_block(&mut self, open: u8, close: u8) -> Result<String> {
        self.expect_byte(open)?;
        let start = self.i;
        let bytes = self.bytes();

        let mut depth = 1usize;
        let mut idx = self.i;
        let mut scanner = JsLexScanner::new();

        while idx < bytes.len() {
            let b = bytes[idx];
            let was_normal = scanner.in_normal();
            idx = scanner.advance(bytes, idx);
            if was_normal {
                if b == open {
                    depth += 1;
                } else if b == close {
                    depth -= 1;
                    if depth == 0 {
                        let body = self
                            .src
                            .get(start..idx - 1)
                            .ok_or_else(|| Error::ScriptParse("invalid block".into()))?
                            .to_string();
                        self.i = idx;
                        return Ok(body);
                    }
                }
            }
        }

        Err(Error::ScriptParse("unclosed block".into()))
    }
}
