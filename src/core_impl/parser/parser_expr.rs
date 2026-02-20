use super::super::html::can_start_regex_literal;
use super::parser_stmt::{
    is_ident_char, parse_callback, parse_element_target, parse_form_elements_base,
    parse_set_interval_call, parse_set_timeout_call, parse_timer_callback,
};
use super::*;

include!("parser_expr_modules/binary_operators.rs");
include!("parser_expr_modules/primary_literals_and_document.rs");
include!("parser_expr_modules/regex_and_date_constructors.rs");
include!("parser_expr_modules/intl_expression_roots.rs");
include!("parser_expr_modules/math_expression_roots.rs");
include!("parser_expr_modules/builtin_constructors.rs");
include!("parser_expr_modules/global_value_accessors.rs");
include!("parser_expr_modules/object_member_and_calls.rs");
include!("parser_expr_modules/array_and_prompt_expr.rs");
include!("parser_expr_modules/intl_string_date_methods.rs");
include!("parser_expr_modules/timers_microtasks_dom_expr.rs");
include!("parser_expr_modules/form_data_assignment_utils.rs");
include!("parser_expr_modules/top_level_split_utils.rs");
