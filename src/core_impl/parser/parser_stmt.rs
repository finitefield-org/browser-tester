use super::api::{
    append_concat_expr, find_first_top_level_colon, find_top_level_assignment, parse_dom_access,
    parse_expr, parse_queue_microtask_stmt, parse_string_literal_exact, split_top_level_by_char,
    strip_js_comments, strip_outer_parens,
};
use super::*;

#[path = "parser_stmt_modules/callback_and_function_parsing.rs"]
pub(super) mod callback_and_function_parsing;
#[path = "parser_stmt_modules/control_flow_statements.rs"]
pub(super) mod control_flow_statements;
#[path = "parser_stmt_modules/dom_assignment_and_object_ops.rs"]
pub(super) mod dom_assignment_and_object_ops;
#[path = "parser_stmt_modules/dom_events_and_listeners.rs"]
pub(super) mod dom_events_and_listeners;
#[path = "parser_stmt_modules/dom_query_targets.rs"]
pub(super) mod dom_query_targets;
#[path = "parser_stmt_modules/foreach_attributes_and_insert_adjacent.rs"]
pub(super) mod foreach_attributes_and_insert_adjacent;
#[path = "parser_stmt_modules/statement_split_and_declarations.rs"]
pub(super) mod statement_split_and_declarations;
#[path = "parser_stmt_modules/timer_statements.rs"]
pub(super) mod timer_statements;

pub(super) use callback_and_function_parsing::*;
pub(super) use control_flow_statements::*;
pub(super) use dom_assignment_and_object_ops::*;
pub(super) use dom_events_and_listeners::*;
pub(super) use dom_query_targets::*;
pub(super) use foreach_attributes_and_insert_adjacent::*;
pub(super) use statement_split_and_declarations::*;
pub(super) use timer_statements::*;
