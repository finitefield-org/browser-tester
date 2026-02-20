use super::form_controls::{is_checkbox_input, is_form_control, is_radio_input, is_submit_control};
use super::html::parse_html;
use super::parser::{
    is_ident, parse_block_statements, parse_expr, resolve_insert_adjacent_position,
};
use super::*;

mod runtime_exec;
mod runtime_platform;
