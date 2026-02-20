use super::super::html::can_start_regex_literal;
use super::parser_stmt::{
    is_ident_char, parse_callback, parse_element_target, parse_form_elements_base,
    parse_set_interval_call, parse_set_timeout_call, parse_timer_callback,
};
use super::*;

include!("parser_expr_parts/part_01.rs");
include!("parser_expr_parts/part_02.rs");
include!("parser_expr_parts/part_03.rs");
include!("parser_expr_parts/part_04.rs");
include!("parser_expr_parts/part_05.rs");
include!("parser_expr_parts/part_06.rs");
include!("parser_expr_parts/part_07.rs");
include!("parser_expr_parts/part_08.rs");
include!("parser_expr_parts/part_09.rs");
include!("parser_expr_parts/part_10.rs");
