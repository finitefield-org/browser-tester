use super::super::html::can_start_regex_literal;
use super::api::{parse_block_statements, parse_function_expr};
use super::parser_stmt::{
    is_ident_char, parse_callback, parse_element_target, parse_form_elements_base,
    parse_set_interval_call, parse_set_timeout_call, parse_timer_callback,
};
use super::*;

#[path = "parser_expr_modules/array_and_prompt_expr.rs"]
pub(super) mod array_and_prompt_expr;
#[path = "parser_expr_modules/binary_operators.rs"]
pub(super) mod binary_operators;
#[path = "parser_expr_modules/builtin_constructors.rs"]
pub(super) mod builtin_constructors;
#[path = "parser_expr_modules/form_data_assignment_utils.rs"]
pub(super) mod form_data_assignment_utils;
#[path = "parser_expr_modules/global_value_accessors.rs"]
pub(super) mod global_value_accessors;
#[path = "parser_expr_modules/intl_expression_roots.rs"]
pub(super) mod intl_expression_roots;
#[path = "parser_expr_modules/intl_string_date_methods.rs"]
pub(super) mod intl_string_date_methods;
#[path = "parser_expr_modules/math_expression_roots.rs"]
pub(super) mod math_expression_roots;
#[path = "parser_expr_modules/object_member_and_calls.rs"]
pub(super) mod object_member_and_calls;
#[path = "parser_expr_modules/primary_literals_and_document.rs"]
pub(super) mod primary_literals_and_document;
#[path = "parser_expr_modules/regex_and_date_constructors.rs"]
pub(super) mod regex_and_date_constructors;
#[path = "parser_expr_modules/timers_microtasks_dom_expr.rs"]
pub(super) mod timers_microtasks_dom_expr;
#[path = "parser_expr_modules/top_level_split_utils.rs"]
pub(super) mod top_level_split_utils;

pub(super) use array_and_prompt_expr::*;
pub(super) use binary_operators::*;
pub(super) use builtin_constructors::*;
pub(super) use form_data_assignment_utils::*;
pub(super) use global_value_accessors::*;
pub(super) use intl_expression_roots::*;
pub(super) use intl_string_date_methods::*;
pub(super) use math_expression_roots::*;
pub(super) use object_member_and_calls::*;
pub(super) use primary_literals_and_document::*;
pub(super) use regex_and_date_constructors::*;
pub(super) use timers_microtasks_dom_expr::*;
pub(super) use top_level_split_utils::*;
