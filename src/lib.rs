//! Deterministic in-memory browser runtime for Rust tests.
//!
//! This crate provides a lightweight DOM and JavaScript-like runtime tailored
//! for deterministic unit and integration testing.
//! Use [`Harness`] as the main entry point to load HTML, simulate user actions,
//! control fake time, and assert DOM state.

use js_regex::{Captures, Regex, RegexBuilder, RegexError, escape as regex_escape};
use num_bigint::{BigInt as JsBigInt, Sign};
use num_traits::{One, ToPrimitive, Zero};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::sync::Arc;

#[path = "lib_parts/core_dom_utils.rs"]
mod core_dom_utils;
#[path = "lib_parts/selector.rs"]
mod selector;
#[path = "lib_parts/runtime_values.rs"]
mod runtime_values;
#[path = "lib_parts/script_ast.rs"]
mod script_ast;
#[path = "lib_parts/runtime_state.rs"]
mod runtime_state;
#[path = "lib_parts/harness_api.rs"]
mod harness_api;

pub use core_dom_utils::{Error, Result, ThrownValue};
pub use harness_api::{Harness, MockPage, MockWindow};
pub use runtime_state::{LocationNavigation, LocationNavigationKind, PendingTimer};

pub(crate) use core_dom_utils::*;
pub(crate) use runtime_state::*;
pub(crate) use runtime_values::*;
pub(crate) use script_ast::*;
pub(crate) use selector::*;

mod core_impl;
mod js_regex;

#[cfg(test)]
fn parse_html(html: &str) -> Result<ParseOutput> {
    core_impl::parse_html(html)
}

#[cfg(test)]
fn parse_for_each_callback(src: &str) -> Result<(String, Option<String>, Vec<Stmt>)> {
    core_impl::parse_for_each_callback(src)
}

fn unescape_string(src: &str) -> String {
    core_impl::unescape_string(src)
}

#[cfg(test)]
mod tests;
