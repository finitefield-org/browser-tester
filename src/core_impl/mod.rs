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
    parser::api::parse_for_each_callback(src)
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    pub(crate) fn parse_expr(src: &str) -> Result<Expr> {
        super::parser::api::parse_expr(src)
    }

    pub(crate) fn parse_block_statements(src: &str) -> Result<Vec<Stmt>> {
        super::parser::api::parse_block_statements(src)
    }
}

pub(super) fn unescape_string(src: &str) -> String {
    html::unescape_string(src)
}
