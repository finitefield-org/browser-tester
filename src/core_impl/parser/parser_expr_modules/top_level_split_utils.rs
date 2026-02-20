use super::*;

pub(crate) fn split_top_level_by_char(src: &str, target: u8) -> Vec<&str> {
    let bytes = src.as_bytes();
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        let b = bytes[i];
        if scanner.is_top_level() && b == target {
            if let Some(part) = src.get(start..i) {
                parts.push(part);
            }
            start = i + 1;
        }
        i = scanner.advance(bytes, i);
    }

    if let Some(last) = src.get(start..) {
        parts.push(last);
    }

    parts
}

pub(crate) fn split_top_level_by_ops<'a>(
    src: &'a str,
    ops: &[&'a str],
) -> (Vec<&'a str>, Vec<&'a str>) {
    let bytes = src.as_bytes();
    let mut parts = Vec::new();
    let mut found_ops = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        if scanner.is_top_level() {
            let mut matched = None;
            for op in ops {
                let op_bytes = op.as_bytes();
                if i + op_bytes.len() <= bytes.len() && &bytes[i..i + op_bytes.len()] == op_bytes {
                    if op_bytes.iter().all(|b| b.is_ascii_alphabetic()) {
                        if i > 0 && is_ident_char(bytes[i - 1]) {
                            continue;
                        }
                        if i + op_bytes.len() < bytes.len()
                            && is_ident_char(bytes[i + op_bytes.len()])
                        {
                            continue;
                        }
                    } else if op.len() == 1 && (op == &"<" || op == &">") {
                        let prev = if i == 0 { None } else { Some(bytes[i - 1]) };
                        let next = bytes.get(i + 1).copied();
                        if prev == Some(b'<')
                            || prev == Some(b'>')
                            || next == Some(b'<')
                            || next == Some(b'>')
                        {
                            continue;
                        }
                    }
                    matched = Some(*op);
                    break;
                }
            }
            if let Some(op) = matched {
                if let Some(part) = src.get(start..i) {
                    parts.push(part);
                    found_ops.push(op);
                    scanner.consume_significant_bytes(b"=");
                    i += op.len();
                    start = i;
                    continue;
                }
            }
        }
        i = scanner.advance(bytes, i);
    }

    if let Some(last) = src.get(start..) {
        parts.push(last);
    }

    (parts, found_ops)
}

pub(crate) fn find_matching_brace(src: &str, start: usize) -> Result<usize> {
    let bytes = src.as_bytes();
    let mut i = start;
    let mut scanner = JsLexScanner {
        mode: JsLexMode::TemplateExpr { brace_depth: 1 },
        mode_stack: vec![JsLexMode::Backtick],
        paren: 0,
        bracket: 0,
        brace: 0,
        previous_significant: None,
        previous_identifier_allows_regex: false,
    };

    while i < bytes.len() {
        let b = bytes[i];
        let before_mode = scanner.mode;
        i = scanner.advance(bytes, i);

        if b == b'}'
            && matches!(before_mode, JsLexMode::TemplateExpr { brace_depth: 1 })
            && matches!(scanner.mode, JsLexMode::Backtick)
            && scanner.mode_stack.is_empty()
        {
            return Ok(i - 1);
        }
    }

    Err(Error::ScriptParse("unclosed template expression".into()))
}
