use super::super::html::can_start_regex_literal;
use super::api::{parse_block_statements, parse_function_expr};
use super::parser_stmt::{
    is_ident_char, parse_callback, parse_element_target, parse_form_elements_base,
    parse_set_interval_call, parse_set_timeout_call, parse_timer_callback,
};
use super::*;

#[path = "parser_expr_modules/arithmetic_expressions.rs"]
pub(super) mod arithmetic_expressions;
#[path = "parser_expr_modules/array_expressions.rs"]
pub(super) mod array_expressions;
#[path = "parser_expr_modules/binary_operators.rs"]
pub(super) mod binary_operators;
#[path = "parser_expr_modules/builtin_constructors.rs"]
pub(super) mod builtin_constructors;
#[path = "parser_expr_modules/dialog_expressions.rs"]
pub(super) mod dialog_expressions;
#[path = "parser_expr_modules/document_navigation_expressions.rs"]
pub(super) mod document_navigation_expressions;
#[path = "parser_expr_modules/form_data_assignment_utils.rs"]
pub(super) mod form_data_assignment_utils;
#[path = "parser_expr_modules/global_numeric_json_expressions.rs"]
pub(super) mod global_numeric_json_expressions;
#[path = "parser_expr_modules/global_value_accessors.rs"]
pub(super) mod global_value_accessors;
#[path = "parser_expr_modules/intl_expression_roots.rs"]
pub(super) mod intl_expression_roots;
#[path = "parser_expr_modules/intl_string_date_methods.rs"]
pub(super) mod intl_string_date_methods;
#[path = "parser_expr_modules/math_expression_roots.rs"]
pub(super) mod math_expression_roots;
#[path = "parser_expr_modules/member_call_expressions.rs"]
pub(super) mod member_call_expressions;
#[path = "parser_expr_modules/numeric_method_expressions.rs"]
pub(super) mod numeric_method_expressions;
#[path = "parser_expr_modules/object_expressions.rs"]
pub(super) mod object_expressions;
#[path = "parser_expr_modules/primary_literal_expressions.rs"]
pub(super) mod primary_literal_expressions;
#[path = "parser_expr_modules/regex_date_constructors.rs"]
pub(super) mod regex_date_constructors;
#[path = "parser_expr_modules/timers_microtasks_dom_expr.rs"]
pub(super) mod timers_microtasks_dom_expr;
#[path = "parser_expr_modules/top_level_split_utils.rs"]
pub(super) mod top_level_split_utils;

pub(super) use arithmetic_expressions::*;
pub(super) use array_expressions::*;
pub(super) use binary_operators::*;
pub(super) use builtin_constructors::*;
pub(super) use dialog_expressions::*;
pub(super) use document_navigation_expressions::*;
pub(super) use form_data_assignment_utils::*;
pub(super) use global_numeric_json_expressions::*;
pub(super) use global_value_accessors::*;
pub(super) use intl_expression_roots::*;
pub(super) use intl_string_date_methods::*;
pub(super) use math_expression_roots::*;
pub(super) use member_call_expressions::*;
pub(super) use numeric_method_expressions::*;
pub(super) use object_expressions::*;
pub(super) use primary_literal_expressions::*;
pub(super) use regex_date_constructors::*;
pub(super) use timers_microtasks_dom_expr::*;
pub(super) use top_level_split_utils::*;
