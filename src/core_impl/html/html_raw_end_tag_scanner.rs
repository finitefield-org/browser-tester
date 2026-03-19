use super::*;

pub(super) fn find_case_insensitive_end_tag(
    bytes: &[u8],
    from: usize,
    tag: &[u8],
) -> Option<usize> {
    fn is_ident_separator(byte: u8) -> bool {
        !byte.is_ascii_alphanumeric()
    }

    fn is_ident_char(byte: u8) -> bool {
        byte == b'_' || byte == b'$' || byte.is_ascii_alphanumeric()
    }

    let mut i = from;
    #[derive(Clone, Copy)]
    enum State {
        Normal,
        Single,
        Double,
        TemplateText,
        TemplateExpr { brace_depth: usize },
        Regex { in_class: bool },
    }
    let mut state_stack = vec![State::Normal];
    let mut previous_significant = None;
    let mut previous_identifier_allows_regex = false;

    while i < bytes.len() {
        let b = bytes[i];

        match state_stack.last().copied().unwrap_or(State::Normal) {
            State::Normal => {
                if b.is_ascii_whitespace() {
                    i += 1;
                    continue;
                }
                if b == b'_' || b == b'$' || b.is_ascii_alphabetic() {
                    let start = i;
                    i += 1;
                    while i < bytes.len() && is_ident_char(bytes[i]) {
                        i += 1;
                    }
                    let prev = previous_significant;
                    previous_significant = Some(bytes[i - 1]);
                    previous_identifier_allows_regex =
                        super::parser::identifier_allows_regex_start(&bytes[start..i], prev);
                    continue;
                }
                if b == b'\'' {
                    state_stack.push(State::Single);
                    previous_identifier_allows_regex = false;
                    i += 1;
                    continue;
                }
                if b == b'"' {
                    state_stack.push(State::Double);
                    previous_identifier_allows_regex = false;
                    i += 1;
                    continue;
                }
                if b == b'`' {
                    state_stack.push(State::TemplateText);
                    previous_identifier_allows_regex = false;
                    i += 1;
                    continue;
                }
                if i + 1 < bytes.len() && b == b'/' && bytes[i + 1] == b'/' {
                    i += 2;
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                    continue;
                }
                if i + 1 < bytes.len() && b == b'/' && bytes[i + 1] == b'*' {
                    i += 2;
                    while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                        i += 1;
                    }
                    if i + 1 < bytes.len() {
                        i += 2;
                    } else {
                        i = bytes.len();
                    }
                    continue;
                }
                if b == b'/' {
                    if can_start_regex_literal(previous_significant)
                        || previous_identifier_allows_regex
                    {
                        state_stack.push(State::Regex { in_class: false });
                        previous_identifier_allows_regex = false;
                        i += 1;
                        continue;
                    }
                    previous_significant = Some(b);
                    previous_identifier_allows_regex = false;
                    i += 1;
                    continue;
                }
                if b == b'<' && bytes.get(i + 1) == Some(&b'/') {
                    let mut j = i + 2;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    let tag_end = j + tag.len();
                    if tag_end <= bytes.len() {
                        if bytes[j..tag_end].eq_ignore_ascii_case(tag) {
                            let after = j + tag.len();
                            if after >= bytes.len() || is_ident_separator(bytes[after]) {
                                return Some(i);
                            }
                        }
                    }
                }
                previous_significant = Some(b);
                previous_identifier_allows_regex = false;
                i += 1;
            }
            State::Single => {
                if b == b'\\' {
                    i += 2;
                } else {
                    if b == b'\'' {
                        state_stack.pop();
                        previous_significant = Some(b'\'');
                        previous_identifier_allows_regex = false;
                    }
                    i += 1;
                }
            }
            State::Double => {
                if b == b'\\' {
                    i += 2;
                } else {
                    if b == b'"' {
                        state_stack.pop();
                        previous_significant = Some(b'"');
                        previous_identifier_allows_regex = false;
                    }
                    i += 1;
                }
            }
            State::TemplateText => {
                if b == b'\\' {
                    i += 2;
                } else {
                    if b == b'`' {
                        state_stack.pop();
                        previous_significant = Some(b'`');
                        previous_identifier_allows_regex = false;
                        i += 1;
                        continue;
                    }
                    if b == b'$' && bytes.get(i + 1) == Some(&b'{') {
                        state_stack.push(State::TemplateExpr { brace_depth: 1 });
                        previous_significant = None;
                        previous_identifier_allows_regex = false;
                        i += 2;
                        continue;
                    }
                    i += 1;
                }
            }
            State::TemplateExpr { brace_depth } => {
                if b.is_ascii_whitespace() {
                    i += 1;
                    continue;
                }
                if b == b'_' || b == b'$' || b.is_ascii_alphabetic() {
                    let start = i;
                    i += 1;
                    while i < bytes.len() && is_ident_char(bytes[i]) {
                        i += 1;
                    }
                    let prev = previous_significant;
                    previous_significant = Some(bytes[i - 1]);
                    previous_identifier_allows_regex =
                        super::parser::identifier_allows_regex_start(&bytes[start..i], prev);
                    continue;
                }
                if b == b'{' {
                    if let Some(State::TemplateExpr { brace_depth }) = state_stack.last_mut() {
                        *brace_depth += 1;
                    }
                    previous_significant = Some(b'{');
                    previous_identifier_allows_regex = false;
                    i += 1;
                    continue;
                }
                if b == b'}' {
                    if brace_depth <= 1 {
                        state_stack.pop();
                    } else if let Some(State::TemplateExpr { brace_depth }) = state_stack.last_mut()
                    {
                        *brace_depth -= 1;
                    }
                    previous_significant = Some(b'}');
                    previous_identifier_allows_regex = false;
                    i += 1;
                    continue;
                }
                if b == b'\'' {
                    state_stack.push(State::Single);
                    previous_identifier_allows_regex = false;
                    i += 1;
                    continue;
                }
                if b == b'"' {
                    state_stack.push(State::Double);
                    previous_identifier_allows_regex = false;
                    i += 1;
                    continue;
                }
                if b == b'`' {
                    state_stack.push(State::TemplateText);
                    previous_identifier_allows_regex = false;
                    i += 1;
                    continue;
                }
                if i + 1 < bytes.len() && b == b'/' && bytes[i + 1] == b'/' {
                    i += 2;
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                    continue;
                }
                if i + 1 < bytes.len() && b == b'/' && bytes[i + 1] == b'*' {
                    i += 2;
                    while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                        i += 1;
                    }
                    if i + 1 < bytes.len() {
                        i += 2;
                    } else {
                        i = bytes.len();
                    }
                    continue;
                }
                if b == b'/' {
                    if can_start_regex_literal(previous_significant)
                        || previous_identifier_allows_regex
                    {
                        state_stack.push(State::Regex { in_class: false });
                        previous_identifier_allows_regex = false;
                        i += 1;
                        continue;
                    }
                    previous_significant = Some(b'/');
                    previous_identifier_allows_regex = false;
                    i += 1;
                    continue;
                }
                previous_significant = Some(b);
                previous_identifier_allows_regex = false;
                i += 1;
            }
            State::Regex { in_class } => {
                if b == b'\\' {
                    i += 2;
                    continue;
                }
                if b == b'[' {
                    if let Some(State::Regex { in_class }) = state_stack.last_mut() {
                        *in_class = true;
                    }
                    i += 1;
                    continue;
                }
                if b == b']' && in_class {
                    if let Some(State::Regex { in_class }) = state_stack.last_mut() {
                        *in_class = false;
                    }
                    i += 1;
                    continue;
                }
                if b == b'/' && !in_class {
                    i += 1;
                    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                        i += 1;
                    }
                    state_stack.pop();
                    previous_significant = Some(b'/');
                    previous_identifier_allows_regex = false;
                    continue;
                }
                i += 1;
            }
        }
    }
    None
}

pub(super) fn find_case_insensitive_raw_end_tag(
    bytes: &[u8],
    from: usize,
    tag: &[u8],
) -> Option<usize> {
    fn is_ident_separator(byte: u8) -> bool {
        !byte.is_ascii_alphanumeric()
    }

    let mut i = from;
    while i < bytes.len() {
        if bytes[i] == b'<' && bytes.get(i + 1) == Some(&b'/') {
            let mut j = i + 2;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            let tag_end = j + tag.len();
            if tag_end <= bytes.len() && bytes[j..tag_end].eq_ignore_ascii_case(tag) {
                let after = j + tag.len();
                if after >= bytes.len() || is_ident_separator(bytes[after]) {
                    return Some(i);
                }
            }
        }
        i += 1;
    }
    None
}
