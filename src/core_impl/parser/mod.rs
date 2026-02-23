use super::*;

mod cursor;
mod ident;
mod js_lex;
mod parser_expr;
mod parser_stmt;

pub(super) use cursor::Cursor;
pub(super) use ident::{identifier_allows_regex_start, is_ident};
pub(super) use js_lex::{JsLexMode, JsLexScanner};

pub(super) mod api {
    use super::*;

    pub(crate) fn parse_expr(src: &str) -> Result<Expr> {
        super::parser_expr::binary_operators::parse_expr(src)
    }

    pub(crate) fn parse_function_expr(src: &str) -> Result<Option<Expr>> {
        super::parser_stmt::callback_expression_parsing::parse_function_expr(src)
    }

    pub(crate) fn parse_class_expr(src: &str) -> Result<Option<Expr>> {
        super::parser_stmt::declaration_assignment_statements::parse_class_expr(src)
    }

    pub(crate) fn parse_block_statements(body: &str) -> Result<Vec<Stmt>> {
        super::parser_stmt::control_flow_statements::parse_block_statements(body)
    }

    pub(crate) fn parse_module_block_statements(body: &str) -> Result<Vec<Stmt>> {
        super::parser_stmt::control_flow_statements::parse_module_block_statements(body)
    }

    #[cfg(test)]
    pub(crate) fn parse_for_each_callback(
        src: &str,
    ) -> Result<(String, Option<String>, Vec<Stmt>)> {
        super::parser_stmt::foreach_statements::parse_for_each_callback(src)
    }

    pub(crate) fn resolve_insert_adjacent_position(src: &str) -> Result<InsertAdjacentPosition> {
        super::parser_stmt::insert_adjacent_statements::resolve_insert_adjacent_position(src)
    }

    pub(crate) fn strip_js_comments(src: &str) -> String {
        super::parser_expr::binary_operators::strip_js_comments(src)
    }

    pub(crate) fn append_concat_expr(lhs: Expr, rhs: Expr) -> Expr {
        super::parser_expr::binary_operators::append_concat_expr(lhs, rhs)
    }

    pub(crate) fn find_first_top_level_colon(src: &str) -> Option<usize> {
        super::parser_expr::object_expressions::find_first_top_level_colon(src)
    }

    pub(crate) fn parse_queue_microtask_stmt(stmt: &str) -> Result<Option<Stmt>> {
        super::parser_expr::timers_microtasks_dom_expr::parse_queue_microtask_stmt(stmt)
    }

    pub(crate) fn parse_dom_access(src: &str) -> Result<Option<(DomQuery, DomProp)>> {
        super::parser_expr::timers_microtasks_dom_expr::parse_dom_access(src)
    }

    pub(crate) fn parse_string_literal_exact(src: &str) -> Result<String> {
        super::parser_expr::expression_syntax_utils::parse_string_literal_exact(src)
    }

    pub(crate) fn strip_outer_parens(src: &str) -> &str {
        super::parser_expr::expression_syntax_utils::strip_outer_parens(src)
    }

    pub(crate) fn find_top_level_assignment(src: &str) -> Option<(usize, usize)> {
        super::parser_expr::expression_syntax_utils::find_top_level_assignment(src)
    }

    pub(crate) fn split_top_level_by_char(src: &str, target: u8) -> Vec<&str> {
        super::parser_expr::top_level_split_utils::split_top_level_by_char(src, target)
    }
}
