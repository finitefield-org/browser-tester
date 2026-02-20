use super::api::{
    append_concat_expr, find_first_top_level_colon, find_top_level_assignment, parse_dom_access,
    parse_expr, parse_queue_microtask_stmt, parse_string_literal_exact, split_top_level_by_char,
    strip_js_comments, strip_outer_parens,
};
use super::*;

#[path = "parser_stmt_modules/attribute_mutation_statements.rs"]
pub(super) mod attribute_mutation_statements;
#[path = "parser_stmt_modules/callback_expression_parsing.rs"]
pub(super) mod callback_expression_parsing;
#[path = "parser_stmt_modules/callback_parameter_parsing.rs"]
pub(super) mod callback_parameter_parsing;
#[path = "parser_stmt_modules/control_flow_statements.rs"]
pub(super) mod control_flow_statements;
#[path = "parser_stmt_modules/declaration_assignment_statements.rs"]
pub(super) mod declaration_assignment_statements;
#[path = "parser_stmt_modules/dom_object_assignment_statements.rs"]
pub(super) mod dom_object_assignment_statements;
#[path = "parser_stmt_modules/dom_query_targets.rs"]
pub(super) mod dom_query_targets;
#[path = "parser_stmt_modules/event_listener_statements.rs"]
pub(super) mod event_listener_statements;
#[path = "parser_stmt_modules/foreach_statements.rs"]
pub(super) mod foreach_statements;
#[path = "parser_stmt_modules/insert_adjacent_statements.rs"]
pub(super) mod insert_adjacent_statements;
#[path = "parser_stmt_modules/node_tree_statements.rs"]
pub(super) mod node_tree_statements;
#[path = "parser_stmt_modules/statement_splitting.rs"]
pub(super) mod statement_splitting;
#[path = "parser_stmt_modules/timer_statements.rs"]
pub(super) mod timer_statements;

pub(super) use attribute_mutation_statements::*;
pub(super) use callback_expression_parsing::*;
pub(super) use callback_parameter_parsing::*;
pub(super) use control_flow_statements::*;
pub(super) use declaration_assignment_statements::*;
pub(super) use dom_object_assignment_statements::*;
pub(super) use dom_query_targets::*;
pub(super) use event_listener_statements::*;
pub(super) use foreach_statements::*;
pub(super) use insert_adjacent_statements::*;
pub(super) use node_tree_statements::*;
pub(super) use statement_splitting::*;
pub(super) use timer_statements::*;
