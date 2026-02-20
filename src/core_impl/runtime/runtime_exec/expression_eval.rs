use super::*;

pub(super) const UNHANDLED_EXPR_CHUNK: &str = "__bt_unhandled_eval_expr_chunk__";

#[path = "expression_eval_modules/bind_and_dispatch.rs"]
mod bind_and_dispatch;
#[path = "expression_eval_modules/calls_timers_binary.rs"]
mod calls_timers_binary;
#[path = "expression_eval_modules/core_date_intl.rs"]
mod core_date_intl;
#[path = "expression_eval_modules/dom_and_platform.rs"]
mod dom_and_platform;
#[path = "expression_eval_modules/events_unary_control.rs"]
mod events_unary_control;
#[path = "expression_eval_modules/json_object_array.rs"]
mod json_object_array;
#[path = "expression_eval_modules/regex_numbers_and_builtins.rs"]
mod regex_numbers_and_builtins;
#[path = "expression_eval_modules/string_and_webapi.rs"]
mod string_and_webapi;
