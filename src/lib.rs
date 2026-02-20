//! Deterministic in-memory browser runtime for Rust tests.
//!
//! This crate provides a lightweight DOM and JavaScript-like runtime tailored
//! for deterministic unit and integration testing.
//! Use [`Harness`] as the main entry point to load HTML, simulate user actions,
//! control fake time, and assert DOM state.

include!("lib_parts/core_dom_utils.rs");
include!("lib_parts/selector.rs");
include!("lib_parts/runtime_values.rs");
include!("lib_parts/script_ast.rs");
include!("lib_parts/runtime_state.rs");
include!("lib_parts/harness_api.rs");

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
