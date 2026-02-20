use super::*;
pub(crate) fn split_top_level_statements(body: &str) -> Vec<String> {
    let bytes = body.as_bytes();
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();
    let mut brace_open_stack = Vec::new();

    while i < bytes.len() {
        let current = i;
        let b = bytes[current];
        let was_normal = scanner.in_normal();
        let paren_before = scanner.paren;
        let bracket_before = scanner.bracket;
        let brace_before = scanner.brace;

        i = scanner.advance(bytes, current);

        if !was_normal {
            continue;
        }

        match b {
            b'{' => {
                brace_open_stack.push(current);
            }
            b'}' => {
                let block_open = brace_open_stack.pop();
                if scanner.is_top_level() {
                    let tail = body.get(i..).unwrap_or_default();
                    if should_split_after_closing_brace(body, block_open, tail) {
                        if let Some(part) = body.get(start..i) {
                            out.push(part.to_string());
                        }
                        start = i;
                    }
                }
            }
            b';' => {
                if paren_before == 0 && bracket_before == 0 && brace_before == 0 {
                    if let Some(part) = body.get(start..current) {
                        out.push(part.to_string());
                    }
                    start = i;
                }
            }
            _ => {}
        }
    }

    if let Some(tail) = body.get(start..) {
        if !tail.trim().is_empty() {
            out.push(tail.to_string());
        }
    }

    out
}

pub(crate) fn should_split_after_closing_brace(
    body: &str,
    block_open: Option<usize>,
    tail: &str,
) -> bool {
    let tail = tail.trim_start();
    if tail.is_empty() {
        return false;
    }
    if tail.starts_with(':') {
        // Keep ternary expressions intact: `cond ? { ... } : { ... }`.
        return false;
    }
    if tail.starts_with('=') {
        // Preserve object destructuring assignment: `{ a, b } = value`.
        return false;
    }
    if is_keyword_prefix(tail, "else") {
        return false;
    }
    if is_keyword_prefix(tail, "catch") {
        return false;
    }
    if is_keyword_prefix(tail, "finally") {
        return false;
    }
    if is_keyword_prefix(tail, "while")
        && block_open.is_some_and(|open| is_do_block_prefix(body, open))
    {
        return false;
    }
    true
}

pub(crate) fn is_do_block_prefix(body: &str, block_open: usize) -> bool {
    let bytes = body.as_bytes();
    if block_open == 0 || block_open > bytes.len() {
        return false;
    }

    let mut j = block_open;
    while j > 0 && bytes[j - 1].is_ascii_whitespace() {
        j -= 1;
    }
    if j < 2 {
        return false;
    }
    if &bytes[j - 2..j] != b"do" {
        return false;
    }
    match bytes.get(j - 3) {
        Some(&b) => !is_ident_char(b),
        None => true,
    }
}

pub(crate) fn is_keyword_prefix(src: &str, keyword: &str) -> bool {
    let Some(rest) = src.strip_prefix(keyword) else {
        return false;
    };
    rest.is_empty() || !is_ident_char(*rest.as_bytes().first().unwrap_or(&b'\0'))
}
