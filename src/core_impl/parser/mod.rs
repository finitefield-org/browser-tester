use super::*;

mod cursor;
mod ident;
mod js_lex;
mod parser_expr;
mod parser_stmt;

pub(super) use cursor::Cursor;
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
