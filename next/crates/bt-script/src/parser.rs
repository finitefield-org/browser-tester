use crate::syntax::{AssignTarget, Expr, Program, Statement};
use crate::{Result, ScriptError, ScriptFunction};

pub(crate) fn parse_program(code: &str) -> Result<Program> {
    let mut parser = Parser::new(code);
    parser.parse_program()
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse_program(&mut self) -> Result<Program> {
        let mut statements = Vec::new();
        self.skip_ws_and_comments();

        while !self.is_eof() {
            statements.push(self.parse_statement()?);
            self.skip_ws_and_comments();
            if self.consume_char(';') {
                self.skip_ws_and_comments();
            }
        }

        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        self.skip_ws_and_comments();
        if self.consume_keyword("const") || self.consume_keyword("let") || self.consume_keyword("var")
        {
            self.skip_ws_and_comments();
            let name = self.parse_identifier()?;
            self.skip_ws_and_comments();
            self.expect_char('=')?;
            let value = self.parse_expression()?;
            return Ok(Statement::VariableDeclaration { name, value });
        }

        let expr = self.parse_expression()?;
        self.skip_ws_and_comments();
        if self.consume_char('=') {
            let value = self.parse_expression()?;
            let target = match expr {
                Expr::Member { object, property } => AssignTarget::Property {
                    object: *object,
                    property,
                },
                _ => {
                    return Err(self.error("unsupported assignment target"));
                }
            };
            Ok(Statement::Assignment { target, value })
        } else {
            Ok(Statement::Expression(expr))
        }
    }

    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<Expr> {
        let mut expr = self.parse_postfix()?;
        loop {
            self.skip_ws_and_comments();
            if self.peek_char() == Some('+') {
                self.pos += 1;
                let rhs = self.parse_postfix()?;
                expr = Expr::BinaryAdd {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            self.skip_ws_and_comments();
            match self.peek_char() {
                Some('.') => {
                    self.pos += 1;
                    let property = self.parse_identifier()?;
                    expr = Expr::Member {
                        object: Box::new(expr),
                        property,
                    };
                }
                Some('(') => {
                    let args = self.parse_call_arguments()?;
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        self.skip_ws_and_comments();
        match self.peek_char() {
            Some('\'') | Some('"') => Ok(Expr::String(self.parse_string()?)),
            Some('(') => {
                if let Some(function) = self.try_parse_arrow_function()? {
                    return Ok(Expr::ArrowFunction(function));
                }

                self.expect_char('(')?;
                let expr = self.parse_expression()?;
                self.skip_ws_and_comments();
                self.expect_char(')')?;
                Ok(expr)
            }
            Some(c) if is_identifier_start(c) => {
                let ident = self.parse_identifier()?;
                Ok(match ident.as_str() {
                    "true" => Expr::Boolean(true),
                    "false" => Expr::Boolean(false),
                    "null" => Expr::Null,
                    "undefined" => Expr::Undefined,
                    _ => Expr::Identifier(ident),
                })
            }
            Some(c) if c.is_ascii_digit() => Ok(Expr::Number(self.parse_number()?)),
            Some(_) => Err(self.error(format!(
                "unsupported syntax near byte {}",
                self.pos
            ))),
            None => Err(self.error("unexpected end of input")),
        }
    }

    fn try_parse_arrow_function(&mut self) -> Result<Option<ScriptFunction>> {
        let start = self.pos;
        if self.peek_char() != Some('(') {
            return Ok(None);
        }

        self.pos += 1;
        self.skip_ws_and_comments();

        let mut params = Vec::new();
        if self.peek_char() != Some(')') {
            loop {
                let param = match self.parse_identifier() {
                    Ok(param) => param,
                    Err(_) => {
                        self.pos = start;
                        return Ok(None);
                    }
                };
                params.push(param);
                self.skip_ws_and_comments();
                if self.consume_char(',') {
                    self.skip_ws_and_comments();
                    continue;
                }
                break;
            }
        }

        self.skip_ws_and_comments();
        if !self.consume_char(')') {
            self.pos = start;
            return Ok(None);
        }
        self.skip_ws_and_comments();
        if !self.consume_str("=>") {
            self.pos = start;
            return Ok(None);
        }
        self.skip_ws_and_comments();
        if self.peek_char() != Some('{') {
            self.pos = start;
            return Ok(None);
        }

        let body_source = self.capture_braced_block()?;
        Ok(Some(ScriptFunction::new(params, body_source)))
    }

    fn parse_call_arguments(&mut self) -> Result<Vec<Expr>> {
        self.expect_char('(')?;
        self.skip_ws_and_comments();

        let mut args = Vec::new();
        if self.consume_char(')') {
            return Ok(args);
        }

        loop {
            let expr = self.parse_expression()?;
            args.push(expr);
            self.skip_ws_and_comments();
            if self.consume_char(')') {
                break;
            }
            self.expect_char(',')?;
            self.skip_ws_and_comments();
        }

        Ok(args)
    }

    fn parse_string(&mut self) -> Result<String> {
        let quote = self
            .bump_char()
            .ok_or_else(|| self.error("unexpected end of input while parsing string"))?;
        let mut out = String::new();

        loop {
            let Some(ch) = self.bump_char() else {
                return Err(self.error("unterminated string literal"));
            };
            if ch == quote {
                break;
            }
            if ch == '\\' {
                let Some(escaped) = self.bump_char() else {
                    return Err(self.error("unterminated escape sequence"));
                };
                out.push(match escaped {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    other => other,
                });
                continue;
            }
            out.push(ch);
        }

        Ok(out)
    }

    fn parse_number(&mut self) -> Result<String> {
        let start = self.pos;
        let mut seen_dot = false;

        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                self.pos += ch.len_utf8();
                continue;
            }
            if ch == '.' && !seen_dot {
                seen_dot = true;
                self.pos += ch.len_utf8();
                continue;
            }
            break;
        }

        if self.pos == start {
            return Err(self.error("expected number"));
        }

        Ok(self.input[start..self.pos].to_string())
    }

    fn parse_identifier(&mut self) -> Result<String> {
        let start = self.pos;
        match self.peek_char() {
            Some(ch) if is_identifier_start(ch) => {
                self.pos += ch.len_utf8();
            }
            _ => return Err(self.error(format!("expected identifier at byte {}", start))),
        }

        while let Some(ch) = self.peek_char() {
            if is_identifier_continue(ch) {
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }

        Ok(self.input[start..self.pos].to_string())
    }

    fn capture_braced_block(&mut self) -> Result<String> {
        self.expect_char('{')?;
        let body_start = self.pos;
        let mut depth = 1usize;
        let mut in_string: Option<char> = None;
        let mut escaped = false;
        let mut in_line_comment = false;
        let mut in_block_comment = false;

        while !self.is_eof() {
            let ch = self.bump_char().expect("not eof");

            if in_line_comment {
                if ch == '\n' {
                    in_line_comment = false;
                }
                continue;
            }

            if in_block_comment {
                if ch == '*' && self.peek_char() == Some('/') {
                    self.pos += 1;
                    in_block_comment = false;
                }
                continue;
            }

            if let Some(quote) = in_string {
                if escaped {
                    escaped = false;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    continue;
                }
                if ch == quote {
                    in_string = None;
                }
                continue;
            }

            match ch {
                '\'' | '"' | '`' => {
                    in_string = Some(ch);
                }
                '/' if self.peek_char() == Some('/') => {
                    self.pos += 1;
                    in_line_comment = true;
                }
                '/' if self.peek_char() == Some('*') => {
                    self.pos += 1;
                    in_block_comment = true;
                }
                '{' => {
                    depth += 1;
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let body_end = self.pos - ch.len_utf8();
                        return Ok(self.input[body_start..body_end].to_string());
                    }
                }
                _ => {}
            }
        }

        Err(self.error("unterminated block body"))
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            let Some(ch) = self.peek_char() else {
                return;
            };

            if ch.is_ascii_whitespace() {
                self.pos += ch.len_utf8();
                continue;
            }

            if ch == '/' && self.peek_next_char() == Some('/') {
                self.pos += 2;
                while let Some(next) = self.peek_char() {
                    self.pos += next.len_utf8();
                    if next == '\n' {
                        break;
                    }
                }
                continue;
            }

            if ch == '/' && self.peek_next_char() == Some('*') {
                self.pos += 2;
                while !self.is_eof() {
                    let Some(next) = self.bump_char() else {
                        break;
                    };
                    if next == '*' && self.peek_char() == Some('/') {
                        self.pos += 1;
                        break;
                    }
                }
                continue;
            }

            break;
        }
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        let start = self.pos;
        if !self.input[start..].starts_with(keyword) {
            return false;
        }

        let end = start + keyword.len();
        let before_ok = start == 0
            || !self
                .input[..start]
                .chars()
                .rev()
                .next()
                .is_some_and(is_identifier_continue);
        let after_ok = self.input[end..]
            .chars()
            .next()
            .map(|ch| !is_identifier_continue(ch))
            .unwrap_or(true);

        if before_ok && after_ok {
            self.pos = end;
            true
        } else {
            false
        }
    }

    fn consume_str(&mut self, expected: &str) -> bool {
        if self.input[self.pos..].starts_with(expected) {
            self.pos += expected.len();
            true
        } else {
            false
        }
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.pos += expected.len_utf8();
            true
        } else {
            false
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<()> {
        if self.consume_char(expected) {
            Ok(())
        } else {
            Err(self.error(format!("expected `{expected}`")))
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn peek_next_char(&self) -> Option<char> {
        let mut chars = self.input[self.pos..].chars();
        chars.next()?;
        chars.next()
    }

    fn bump_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn error(&self, message: impl Into<String>) -> ScriptError {
        ScriptError::new(message)
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '$'
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}
