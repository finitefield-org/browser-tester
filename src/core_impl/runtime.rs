use super::html::parse_html;
use super::parser::{
    is_checkbox_input, is_form_control, is_ident, is_radio_input, is_submit_control,
    parse_block_statements, parse_expr, resolve_insert_adjacent_position,
};
use super::*;

include!("runtime_platform.rs");
include!("runtime_exec.rs");
