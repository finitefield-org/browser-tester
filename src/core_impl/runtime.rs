use super::html::parse_html;
use super::parser::{
    is_checkbox_input, is_form_control, is_ident, is_radio_input, is_submit_control,
    parse_block_statements, parse_expr, resolve_insert_adjacent_position,
};
use super::*;

#[path = "runtime_platform.rs"]
mod runtime_platform;
#[path = "runtime_exec.rs"]
mod runtime_exec;
