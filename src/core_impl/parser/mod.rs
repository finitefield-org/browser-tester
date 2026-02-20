use super::html::unescape_string;
use super::*;

mod ident;
mod js_lex;
mod parser_expr;
mod parser_stmt;

pub(super) use ident::{identifier_allows_regex_start, is_ident};
pub(super) use js_lex::{JsLexMode, JsLexScanner};

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

pub(super) fn strip_js_comments(src: &str) -> String {
    parser_expr::strip_js_comments(src)
}

pub(super) fn append_concat_expr(lhs: Expr, rhs: Expr) -> Expr {
    parser_expr::append_concat_expr(lhs, rhs)
}

pub(super) fn find_first_top_level_colon(src: &str) -> Option<usize> {
    parser_expr::find_first_top_level_colon(src)
}

pub(super) fn parse_queue_microtask_stmt(stmt: &str) -> Result<Option<Stmt>> {
    parser_expr::parse_queue_microtask_stmt(stmt)
}

pub(super) fn parse_dom_access(src: &str) -> Result<Option<(DomQuery, DomProp)>> {
    parser_expr::parse_dom_access(src)
}

pub(super) fn parse_string_literal_exact(src: &str) -> Result<String> {
    parser_expr::parse_string_literal_exact(src)
}

pub(super) fn strip_outer_parens(src: &str) -> &str {
    parser_expr::strip_outer_parens(src)
}

pub(super) fn find_top_level_assignment(src: &str) -> Option<(usize, usize)> {
    parser_expr::find_top_level_assignment(src)
}

pub(super) fn split_top_level_by_char(src: &str, target: u8) -> Vec<&str> {
    parser_expr::split_top_level_by_char(src, target)
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
