use super::super::html::can_start_regex_literal;
use super::identifier_allows_regex_start;
use super::parser_stmt::is_ident_char;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JsLexMode {
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
pub(crate) struct JsLexScanner {
    pub(crate) mode: JsLexMode,
    pub(crate) mode_stack: Vec<JsLexMode>,
    pub(crate) paren: usize,
    pub(crate) bracket: usize,
    pub(crate) brace: usize,
    pub(crate) previous_significant: Option<u8>,
    pub(crate) previous_identifier_allows_regex: bool,
}

impl JsLexScanner {
    pub(crate) fn new() -> Self {
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

    pub(crate) fn in_normal(&self) -> bool {
        matches!(self.mode, JsLexMode::Normal)
    }

    pub(crate) fn is_top_level(&self) -> bool {
        self.in_normal() && self.paren == 0 && self.bracket == 0 && self.brace == 0
    }

    pub(crate) fn consume_significant_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.note_significant_byte(b);
        }
    }

    pub(crate) fn slash_starts_comment_or_regex(&self, bytes: &[u8], i: usize) -> bool {
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

    pub(crate) fn advance(&mut self, bytes: &[u8], i: usize) -> usize {
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
