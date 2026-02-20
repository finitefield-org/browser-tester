use super::*;

mod dom;
mod form_controls;
mod html;
mod intl;
mod parser;
mod runtime;

#[cfg(test)]
pub(super) fn parse_html(html: &str) -> Result<ParseOutput> {
    html::parse_html(html)
}

#[cfg(test)]
pub(super) fn parse_for_each_callback(src: &str) -> Result<(String, Option<String>, Vec<Stmt>)> {
    parser::parse_for_each_callback(src)
}

pub(super) fn unescape_string(src: &str) -> String {
    html::unescape_string(src)
}
