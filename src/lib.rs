//! Deterministic in-memory browser runtime for Rust tests.
//!
//! This crate provides a lightweight DOM and JavaScript-like runtime tailored
//! for deterministic unit and integration testing.
//! Use [`Harness`] as the main entry point to load HTML, simulate user actions,
//! control fake time, and assert DOM state.

use js_regex::{Regex, RegexBuilder, RegexError, escape as regex_escape};
use num_bigint::{BigInt as JsBigInt, Sign};
use num_traits::{One, ToPrimitive, Zero};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::sync::Arc;

mod core_dom_utils;
mod harness_api;
mod runtime_state;
mod runtime_values;
mod script_ast;
mod selector;

pub use core_dom_utils::MockFile;
pub use core_dom_utils::{Error, Result, ThrownValue};
pub use harness_api::{Harness, KeyboardEventInit, MockPage, MockWindow};
pub use runtime_state::{
    ClipboardPayloadArtifact, ClipboardWriteArtifact, DownloadArtifact, LocationNavigation,
    LocationNavigationKind, PendingTimer,
};

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

#[cfg(test)]
mod test_support {
    use super::*;

    pub(crate) fn parse_expr(src: &str) -> Result<Expr> {
        core_impl::test_support::parse_expr(src)
    }

    pub(crate) fn parse_block_statements(src: &str) -> Result<Vec<Stmt>> {
        core_impl::test_support::parse_block_statements(src)
    }
}

fn unescape_string(src: &str) -> String {
    core_impl::unescape_string(src)
}

#[cfg(test)]
mod tests;
