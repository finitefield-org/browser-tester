use crate::syntax::{
    ArrayElement, AssignTarget, AssignmentOperator, ComparisonOperator, Expr, ObjectProperty,
    ObjectPropertyName, Program, Statement,
};
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
        if self.consume_keyword("function") {
            return self.parse_function_declaration();
        }
        if self.consume_keyword("return") {
            return self.parse_return_statement();
        }
        if self.consume_keyword("break") {
            return Ok(Statement::Break);
        }
        if self.consume_keyword("continue") {
            return Ok(Statement::Continue);
        }
        if self.consume_keyword("if") {
            return self.parse_if_statement();
        }
        if self.consume_keyword("while") {
            return self.parse_while_statement();
        }
        if self.consume_keyword("for") {
            return self.parse_for_statement();
        }
        if self.consume_keyword("const")
            || self.consume_keyword("let")
            || self.consume_keyword("var")
        {
            return self.parse_variable_declaration();
        }

        let expr = self.parse_expression()?;
        Ok(Statement::Expression(expr))
    }

    fn assignment_target_from_expr(&self, expr: &Expr) -> Result<AssignTarget> {
        match expr {
            Expr::Identifier(name) => Ok(AssignTarget::Identifier(name.clone())),
            Expr::Member { object, property } => Ok(AssignTarget::Property {
                object: object.clone(),
                property: property.clone(),
            }),
            Expr::ComputedMember { object, property } => Ok(AssignTarget::ComputedProperty {
                object: object.clone(),
                property: property.clone(),
            }),
            _ => Err(self.error("unsupported assignment target")),
        }
    }

    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr> {
        let expr = self.parse_conditional()?;
        self.skip_ws_and_comments();
        if self.consume_str("+=") {
            let value = self.parse_assignment()?;
            let target = self.assignment_target_from_expr(&expr)?;
            return Ok(Expr::Assignment {
                target,
                value: Box::new(value),
                operator: AssignmentOperator::AddAssign,
            });
        }
        if self.consume_char('=') {
            let value = self.parse_assignment()?;
            let target = self.assignment_target_from_expr(&expr)?;
            return Ok(Expr::Assignment {
                target,
                value: Box::new(value),
                operator: AssignmentOperator::Assign,
            });
        }

        Ok(expr)
    }

    fn parse_conditional(&mut self) -> Result<Expr> {
        let condition = self.parse_nullish()?;
        self.skip_ws_and_comments();
        if self.consume_char('?') {
            let consequent = self.parse_expression()?;
            self.skip_ws_and_comments();
            self.expect_char(':')?;
            let alternate = self.parse_expression()?;
            Ok(Expr::Conditional {
                condition: Box::new(condition),
                consequent: Box::new(consequent),
                alternate: Box::new(alternate),
            })
        } else {
            Ok(condition)
        }
    }

    fn parse_nullish(&mut self) -> Result<Expr> {
        let mut expr = self.parse_logical_or()?;
        loop {
            self.skip_ws_and_comments();
            if self.consume_str("??") {
                let rhs = self.parse_logical_or()?;
                expr = Expr::NullishCoalesce {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_logical_and()?;
        loop {
            self.skip_ws_and_comments();
            if self.consume_str("||") {
                let rhs = self.parse_logical_and()?;
                expr = Expr::LogicalOr {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expr> {
        let mut expr = self.parse_equality()?;
        loop {
            self.skip_ws_and_comments();
            if self.consume_str("&&") {
                let rhs = self.parse_equality()?;
                expr = Expr::LogicalAnd {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut expr = self.parse_comparison()?;
        loop {
            self.skip_ws_and_comments();
            if self.consume_str("===") {
                let rhs = self.parse_comparison()?;
                expr = Expr::Equality {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                    negated: false,
                    strict: true,
                };
            } else if self.consume_str("!==") {
                let rhs = self.parse_comparison()?;
                expr = Expr::Equality {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                    negated: true,
                    strict: true,
                };
            } else if self.consume_str("==") {
                let rhs = self.parse_comparison()?;
                expr = Expr::Equality {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                    negated: false,
                    strict: false,
                };
            } else if self.consume_str("!=") {
                let rhs = self.parse_comparison()?;
                expr = Expr::Equality {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                    negated: true,
                    strict: false,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut expr = self.parse_additive()?;
        loop {
            self.skip_ws_and_comments();
            let operator = if self.consume_str("<=") {
                Some(ComparisonOperator::LessThanOrEqual)
            } else if self.consume_str(">=") {
                Some(ComparisonOperator::GreaterThanOrEqual)
            } else if self.consume_char('<') {
                Some(ComparisonOperator::LessThan)
            } else if self.consume_char('>') {
                Some(ComparisonOperator::GreaterThan)
            } else {
                None
            };
            let Some(operator) = operator else {
                break;
            };
            let rhs = self.parse_additive()?;
            expr = Expr::Comparison {
                left: Box::new(expr),
                right: Box::new(rhs),
                operator,
            };
        }
        Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<Expr> {
        let mut expr = self.parse_multiplicative()?;
        loop {
            self.skip_ws_and_comments();
            if self.peek_char() == Some('+') && self.peek_next_char() != Some('=') {
                self.pos += 1;
                let rhs = self.parse_multiplicative()?;
                expr = Expr::BinaryAdd {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
            } else if self.peek_char() == Some('-') {
                self.pos += 1;
                let rhs = self.parse_multiplicative()?;
                expr = Expr::BinarySub {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;
        loop {
            self.skip_ws_and_comments();
            if self.consume_char('*') {
                let rhs = self.parse_unary()?;
                expr = Expr::BinaryMul {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
            } else if self.consume_char('/') {
                let rhs = self.parse_unary()?;
                expr = Expr::BinaryDiv {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
            } else if self.consume_char('%') {
                let rhs = self.parse_unary()?;
                expr = Expr::BinaryRem {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        self.skip_ws_and_comments();
        if self.consume_keyword("new") {
            return self.parse_new_expression();
        }
        if self.consume_str("++") {
            let expr = self.parse_unary()?;
            let target = self.assignment_target_from_expr(&expr)?;
            return Ok(Expr::Update {
                target,
                increment: true,
                prefix: true,
            });
        }
        if self.consume_str("--") {
            let expr = self.parse_unary()?;
            let target = self.assignment_target_from_expr(&expr)?;
            return Ok(Expr::Update {
                target,
                increment: false,
                prefix: true,
            });
        }
        if self.consume_char('!') {
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryNot(Box::new(expr)));
        }
        if self.consume_char('-') {
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryNeg(Box::new(expr)));
        }
        if self.consume_keyword("typeof") {
            let expr = self.parse_unary()?;
            return Ok(Expr::TypeOf(Box::new(expr)));
        }
        if self.consume_keyword("void") {
            let expr = self.parse_unary()?;
            return Ok(Expr::Void(Box::new(expr)));
        }

        self.parse_postfix()
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
                Some('[') => {
                    self.pos += 1;
                    let property = self.parse_expression()?;
                    self.skip_ws_and_comments();
                    self.expect_char(']')?;
                    expr = Expr::ComputedMember {
                        object: Box::new(expr),
                        property: Box::new(property),
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

        self.skip_ws_and_comments();
        if self.consume_str("++") {
            let target = self.assignment_target_from_expr(&expr)?;
            expr = Expr::Update {
                target,
                increment: true,
                prefix: false,
            };
        } else if self.consume_str("--") {
            let target = self.assignment_target_from_expr(&expr)?;
            expr = Expr::Update {
                target,
                increment: false,
                prefix: false,
            };
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        self.skip_ws_and_comments();
        if self.consume_keyword("function") {
            return self.parse_function_expression();
        }
        match self.peek_char() {
            Some('[') => self.parse_array_literal(),
            Some('{') => self.parse_object_literal(),
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
            Some(_) => Err(self.error(format!("unsupported syntax near byte {}", self.pos))),
            None => Err(self.error("unexpected end of input")),
        }
    }

    fn parse_new_expression(&mut self) -> Result<Expr> {
        self.skip_ws_and_comments();
        let callee = self.parse_new_callee()?;
        self.skip_ws_and_comments();
        let args = if self.peek_char() == Some('(') {
            self.parse_call_arguments()?
        } else {
            Vec::new()
        };
        Ok(Expr::New {
            callee: Box::new(callee),
            args,
        })
    }

    fn parse_new_callee(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;
        loop {
            self.skip_ws_and_comments();
            if self.peek_char() != Some('.') {
                break;
            }
            self.pos += 1;
            let property = self.parse_identifier()?;
            expr = Expr::Member {
                object: Box::new(expr),
                property,
            };
        }

        Ok(expr)
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement> {
        self.skip_ws_and_comments();
        let name = self.parse_identifier()?;
        self.skip_ws_and_comments();
        self.expect_char('=')?;
        let value = self.parse_expression()?;
        Ok(Statement::VariableDeclaration { name, value })
    }

    fn parse_return_statement(&mut self) -> Result<Statement> {
        self.skip_ws_and_comments();
        if self.peek_char().is_none() || matches!(self.peek_char(), Some(';') | Some('}')) {
            return Ok(Statement::Return(None));
        }

        let value = self.parse_expression()?;
        Ok(Statement::Return(Some(value)))
    }

    fn parse_if_statement(&mut self) -> Result<Statement> {
        self.skip_ws_and_comments();
        self.expect_char('(')?;
        let condition = self.parse_expression()?;
        self.skip_ws_and_comments();
        self.expect_char(')')?;
        let then_branch = self.parse_statement_body()?;
        self.skip_ws_and_comments();
        let else_branch = if self.consume_keyword("else") {
            Some(self.parse_statement_body()?)
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_while_statement(&mut self) -> Result<Statement> {
        self.skip_ws_and_comments();
        self.expect_char('(')?;
        let condition = self.parse_expression()?;
        self.skip_ws_and_comments();
        self.expect_char(')')?;
        let body = self.parse_statement_body()?;
        Ok(Statement::While { condition, body })
    }

    fn parse_for_statement(&mut self) -> Result<Statement> {
        self.skip_ws_and_comments();
        self.expect_char('(')?;
        self.skip_ws_and_comments();

        let init = if self.consume_char(';') {
            None
        } else if self.consume_keyword("const")
            || self.consume_keyword("let")
            || self.consume_keyword("var")
        {
            let statement = self.parse_variable_declaration()?;
            self.skip_ws_and_comments();
            self.expect_char(';')?;
            Some(Box::new(statement))
        } else {
            let expr = self.parse_expression()?;
            self.skip_ws_and_comments();
            self.expect_char(';')?;
            Some(Box::new(Statement::Expression(expr)))
        };

        self.skip_ws_and_comments();
        let condition = if self.consume_char(';') {
            None
        } else {
            let condition = self.parse_expression()?;
            self.skip_ws_and_comments();
            self.expect_char(';')?;
            Some(condition)
        };

        self.skip_ws_and_comments();
        let update = if self.consume_char(')') {
            None
        } else {
            let update = self.parse_expression()?;
            self.skip_ws_and_comments();
            self.expect_char(')')?;
            Some(update)
        };

        let body = self.parse_statement_body()?;
        Ok(Statement::For {
            init,
            condition,
            update,
            body,
        })
    }

    fn parse_statement_body(&mut self) -> Result<Vec<Statement>> {
        self.skip_ws_and_comments();
        if self.peek_char() == Some('{') {
            let body_source = self.capture_braced_block()?;
            return Ok(parse_program(&body_source)?.statements);
        }

        let statement = self.parse_statement()?;
        self.skip_ws_and_comments();
        let _ = self.consume_char(';');
        Ok(vec![statement])
    }

    fn parse_function_declaration(&mut self) -> Result<Statement> {
        self.skip_ws_and_comments();
        let name = self.parse_identifier()?;
        let function = self.parse_function_like(true)?;
        Ok(Statement::FunctionDeclaration { name, function })
    }

    fn parse_function_expression(&mut self) -> Result<Expr> {
        let function = self.parse_function_like(false)?;
        Ok(Expr::FunctionExpression(function))
    }

    fn parse_array_literal(&mut self) -> Result<Expr> {
        self.expect_char('[')?;
        self.skip_ws_and_comments();

        let mut elements = Vec::new();
        if self.consume_char(']') {
            return Ok(Expr::ArrayLiteral(elements));
        }

        loop {
            self.skip_ws_and_comments();
            if self.consume_str("...") {
                let expr = self.parse_expression()?;
                elements.push(ArrayElement::Spread(expr));
            } else {
                let expr = self.parse_expression()?;
                elements.push(ArrayElement::Expression(expr));
            }
            self.skip_ws_and_comments();
            if self.consume_char(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_ws_and_comments();
            if self.consume_char(']') {
                break;
            }
        }

        Ok(Expr::ArrayLiteral(elements))
    }

    fn parse_object_literal(&mut self) -> Result<Expr> {
        self.expect_char('{')?;
        self.skip_ws_and_comments();

        let mut properties = Vec::new();
        if self.consume_char('}') {
            return Ok(Expr::ObjectLiteral(properties));
        }

        loop {
            self.skip_ws_and_comments();
            if self.consume_str("...") {
                let expr = self.parse_expression()?;
                properties.push(ObjectProperty::Spread(expr));
            } else {
                properties.push(self.parse_object_property()?);
            }
            self.skip_ws_and_comments();
            if self.consume_char('}') {
                break;
            }
            self.expect_char(',')?;
            self.skip_ws_and_comments();
            if self.consume_char('}') {
                break;
            }
        }

        Ok(Expr::ObjectLiteral(properties))
    }

    fn parse_object_property(&mut self) -> Result<ObjectProperty> {
        let property_name = self.parse_object_property_name()?;
        self.skip_ws_and_comments();

        if self.consume_char(':') {
            let value = self.parse_expression()?;
            return Ok(ObjectProperty::KeyValue {
                name: property_name,
                value,
            });
        }

        if let ObjectPropertyName::Static(name) = &property_name {
            if self.peek_char() == Some('(') {
                let function = self.parse_method_like_function()?;
                return Ok(ObjectProperty::Method {
                    name: property_name,
                    function,
                });
            }

            if name == "get" || name == "set" {
                self.skip_ws_and_comments();
                let accessor_name = self.parse_object_property_name()?;
                self.skip_ws_and_comments();
                let function = self.parse_method_like_function()?;
                return Ok(match name.as_str() {
                    "get" => ObjectProperty::Getter {
                        name: accessor_name,
                        function,
                    },
                    "set" => ObjectProperty::Setter {
                        name: accessor_name,
                        function,
                    },
                    _ => unreachable!(),
                });
            }
        }

        if let ObjectPropertyName::Static(name) = property_name {
            return Ok(ObjectProperty::KeyValue {
                name: ObjectPropertyName::Static(name.clone()),
                value: Expr::Identifier(name),
            });
        }

        Err(self.error("unsupported object literal property"))
    }

    fn parse_object_property_name(&mut self) -> Result<ObjectPropertyName> {
        self.skip_ws_and_comments();
        match self.peek_char() {
            Some('\'') | Some('"') => Ok(ObjectPropertyName::Static(self.parse_string()?)),
            Some('[') => {
                self.pos += 1;
                let expr = self.parse_expression()?;
                self.skip_ws_and_comments();
                self.expect_char(']')?;
                Ok(ObjectPropertyName::Computed(expr))
            }
            Some(c) if c.is_ascii_digit() => Ok(ObjectPropertyName::Static(self.parse_number()?)),
            Some(c) if is_identifier_start(c) => {
                Ok(ObjectPropertyName::Static(self.parse_identifier()?))
            }
            Some(_) => Err(self.error(format!(
                "unsupported object literal property near byte {}",
                self.pos
            ))),
            None => Err(self.error("unexpected end of input")),
        }
    }

    fn parse_method_like_function(&mut self) -> Result<ScriptFunction> {
        self.skip_ws_and_comments();
        self.expect_char('(')?;
        let (params, defaults) = self.parse_function_parameters()?;
        self.skip_ws_and_comments();
        let body_source = self.capture_braced_block()?;
        let mut body = String::new();
        for (param, default) in params.iter().zip(defaults.iter()) {
            if let Some(default) = default {
                body.push_str("if (typeof ");
                body.push_str(param);
                body.push_str(" === \"undefined\") { ");
                body.push_str(param);
                body.push_str(" = ");
                body.push_str(default);
                body.push_str("; }\n");
            }
        }
        body.push_str(&body_source);
        Ok(ScriptFunction::new(params, body))
    }

    fn parse_function_like(&mut self, declaration: bool) -> Result<ScriptFunction> {
        self.skip_ws_and_comments();
        if !declaration && self.peek_char().is_some_and(is_identifier_start) {
            let _ = self.parse_identifier()?;
            self.skip_ws_and_comments();
        }

        self.expect_char('(')?;
        let (params, defaults) = self.parse_function_parameters()?;
        self.skip_ws_and_comments();
        let body_source = self.capture_braced_block()?;
        let mut body = String::new();
        for (param, default) in params.iter().zip(defaults.iter()) {
            if let Some(default) = default {
                body.push_str("if (typeof ");
                body.push_str(param);
                body.push_str(" === \"undefined\") { ");
                body.push_str(param);
                body.push_str(" = ");
                body.push_str(default);
                body.push_str("; }\n");
            }
        }
        body.push_str(&body_source);
        Ok(ScriptFunction::new(params, body))
    }

    fn parse_function_parameters(&mut self) -> Result<(Vec<String>, Vec<Option<String>>)> {
        self.skip_ws_and_comments();
        let mut params = Vec::new();
        let mut defaults = Vec::new();

        if self.consume_char(')') {
            return Ok((params, defaults));
        }

        loop {
            let name = self.parse_identifier()?;
            self.skip_ws_and_comments();
            let default = if self.consume_char('=') {
                self.skip_ws_and_comments();
                let source = self.capture_expression_source_until(&[',', ')'])?;
                Some(source.trim().to_string())
            } else {
                None
            };
            params.push(name);
            defaults.push(default);
            self.skip_ws_and_comments();
            if self.consume_char(')') {
                break;
            }
            self.expect_char(',')?;
            self.skip_ws_and_comments();
        }

        Ok((params, defaults))
    }

    fn capture_expression_source_until(&mut self, terminators: &[char]) -> Result<String> {
        let start = self.pos;
        let mut paren_depth = 0usize;
        let mut brace_depth = 0usize;
        let mut bracket_depth = 0usize;
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
                '(' => {
                    paren_depth += 1;
                }
                ')' => {
                    if paren_depth == 0
                        && brace_depth == 0
                        && bracket_depth == 0
                        && terminators.contains(&')')
                    {
                        let end = self.pos - ch.len_utf8();
                        return Ok(self.input[start..end].to_string());
                    }
                    paren_depth = paren_depth.saturating_sub(1);
                }
                '{' => {
                    brace_depth += 1;
                }
                '}' => {
                    if brace_depth == 0
                        && paren_depth == 0
                        && bracket_depth == 0
                        && terminators.contains(&'}')
                    {
                        let end = self.pos - ch.len_utf8();
                        return Ok(self.input[start..end].to_string());
                    }
                    brace_depth = brace_depth.saturating_sub(1);
                }
                '[' => {
                    bracket_depth += 1;
                }
                ']' => {
                    if paren_depth == 0
                        && brace_depth == 0
                        && bracket_depth == 0
                        && terminators.contains(&']')
                    {
                        let end = self.pos - ch.len_utf8();
                        return Ok(self.input[start..end].to_string());
                    }
                    bracket_depth = bracket_depth.saturating_sub(1);
                }
                ',' => {
                    if paren_depth == 0
                        && brace_depth == 0
                        && bracket_depth == 0
                        && terminators.contains(&',')
                    {
                        let end = self.pos - ch.len_utf8();
                        return Ok(self.input[start..end].to_string());
                    }
                }
                _ => {}
            }
        }

        Err(self.error("unterminated expression"))
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
            if self.consume_str("...") {
                let expr = self.parse_expression()?;
                args.push(Expr::Spread(Box::new(expr)));
            } else {
                let expr = self.parse_expression()?;
                args.push(expr);
            }
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
            || !self.input[..start]
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
        ScriptError::parse(message)
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '$'
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}
