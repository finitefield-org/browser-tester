use super::*;
pub(crate) fn parse_string_literal_exact(src: &str) -> Result<String> {
    let bytes = src.as_bytes();
    if bytes.len() < 2 {
        return Err(Error::ScriptParse("invalid string literal".into()));
    }
    let quote = bytes[0];
    if (quote != b'\'' && quote != b'"') || bytes[bytes.len() - 1] != quote {
        return Err(Error::ScriptParse(format!("invalid string literal: {src}")));
    }

    let mut escaped = false;
    let mut i = 1;
    while i + 1 < bytes.len() {
        let b = bytes[i];
        if escaped {
            escaped = false;
        } else if b == b'\\' {
            escaped = true;
        } else if b == quote {
            return Err(Error::ScriptParse(format!("unexpected quote in: {src}")));
        }
        i += 1;
    }

    Ok(unescape_string(&src[1..src.len() - 1]))
}

pub(crate) fn strip_outer_parens(mut src: &str) -> &str {
    loop {
        let trimmed = src.trim();
        if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
            return trimmed;
        }

        if !is_fully_wrapped_in_parens(trimmed) {
            return trimmed;
        }

        src = &trimmed[1..trimmed.len() - 1];
    }
}

pub(crate) fn is_fully_wrapped_in_parens(src: &str) -> bool {
    let bytes = src.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'(' || bytes[bytes.len() - 1] != b')' {
        return false;
    }

    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        if scanner.in_normal() && bytes[i] == b')' && scanner.paren == 1 {
            let mut tail = i + 1;
            while tail < bytes.len() && bytes[tail].is_ascii_whitespace() {
                tail += 1;
            }
            if tail < bytes.len() {
                return false;
            }
        }
        i = scanner.advance(bytes, i);
    }

    scanner.in_normal() && scanner.paren == 0 && scanner.bracket == 0 && scanner.brace == 0
}

pub(crate) fn find_top_level_assignment(src: &str) -> Option<(usize, usize)> {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        if scanner.is_top_level() && bytes[i] == b'=' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                i = scanner.advance(bytes, i);
                continue;
            }
            if i + 1 < bytes.len() && bytes[i + 1] == b'>' {
                i = scanner.advance(bytes, i);
                continue;
            }
            if i >= 3 && &bytes[i - 3..=i] == b">>>=" {
                return Some((i - 3, 4));
            }
            if i >= 2
                && matches!(
                    &bytes[i - 2..=i],
                    b"&&=" | b"||=" | b"??=" | b"**=" | b"<<=" | b">>="
                )
            {
                return Some((i - 2, 3));
            }
            if i > 0 {
                let prev = bytes[i - 1];
                if matches!(prev, b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|' | b'^') {
                    return Some((i - 1, 2));
                }
                if matches!(prev, b'!' | b'<' | b'>' | b'=') {
                    i = scanner.advance(bytes, i);
                    continue;
                }
            }
            return Some((i, 1));
        }
        i = scanner.advance(bytes, i);
    }

    None
}

pub(crate) fn find_top_level_ternary_question(src: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        if scanner.is_top_level() && bytes[i] == b'?' {
            if i + 1 < bytes.len() && (bytes[i + 1] == b'?' || bytes[i + 1] == b'.') {
                i = scanner.advance(bytes, i);
                continue;
            }
            if i > 0 && bytes[i - 1] == b'?' {
                i = scanner.advance(bytes, i);
                continue;
            }
            return Some(i);
        }
        i = scanner.advance(bytes, i);
    }

    None
}

pub(crate) fn find_matching_ternary_colon(src: &str, from: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    if from >= bytes.len() {
        return None;
    }

    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();
    let mut nested_ternary = 0usize;

    while i < bytes.len() {
        if i >= from && scanner.is_top_level() {
            let b = bytes[i];
            if b == b'?' {
                if i + 1 < bytes.len() && (bytes[i + 1] == b'?' || bytes[i + 1] == b'.') {
                    i = scanner.advance(bytes, i);
                    continue;
                }
                if i > 0 && bytes[i - 1] == b'?' {
                    i = scanner.advance(bytes, i);
                    continue;
                }
                nested_ternary += 1;
            } else if b == b':' {
                if nested_ternary == 0 {
                    return Some(i);
                }
                nested_ternary -= 1;
            }
        }
        i = scanner.advance(bytes, i);
    }

    None
}
