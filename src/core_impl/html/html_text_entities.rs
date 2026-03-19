use super::*;

pub(super) fn decode_html_character_references(src: &str) -> String {
    if !src.contains('&') {
        return src.to_string();
    }

    fn is_entity_token_char(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '#' || ch == 'x' || ch == 'X'
    }

    fn decode_numeric(value: &str) -> Option<char> {
        let codepoint =
            if let Some(hex) = value.strip_prefix("x").or_else(|| value.strip_prefix("X")) {
                u32::from_str_radix(hex, 16).ok()?
            } else {
                u32::from_str_radix(value, 10).ok()?
            };
        char::from_u32(codepoint)
    }

    fn decode_named(value: &str) -> Option<char> {
        match value {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" => Some('\u{00A0}'),
            "divide" => Some('÷'),
            "times" => Some('×'),
            "ensp" => Some('\u{2002}'),
            "emsp" => Some('\u{2003}'),
            "thinsp" => Some('\u{2009}'),
            "copy" => Some('©'),
            "reg" => Some('®'),
            "trade" => Some('™'),
            "euro" => Some('€'),
            "pound" => Some('£'),
            "yen" => Some('¥'),
            "laquo" => Some('«'),
            "raquo" => Some('»'),
            "ldquo" => Some('“'),
            "rdquo" => Some('”'),
            "lsquo" => Some('‘'),
            "rsquo" => Some('’'),
            "hellip" => Some('…'),
            "middot" => Some('·'),
            "frac14" => Some('¼'),
            "frac12" => Some('½'),
            "frac34" => Some('¾'),
            "frac13" => Some('\u{2153}'),
            "frac15" => Some('\u{2155}'),
            "frac16" => Some('\u{2159}'),
            "frac18" => Some('\u{215B}'),
            "frac23" => Some('\u{2154}'),
            "frac25" => Some('\u{2156}'),
            "frac35" => Some('\u{2157}'),
            "frac38" => Some('\u{215C}'),
            "frac45" => Some('\u{2158}'),
            "frac56" => Some('\u{215A}'),
            "frac58" => Some('\u{215E}'),
            "not" => Some('¬'),
            "deg" => Some('°'),
            "plusmn" => Some('±'),
            "larr" => Some('←'),
            "rarr" => Some('→'),
            _ => None,
        }
    }

    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;

    while i < src.len() {
        let ch = src[i..].chars().next().unwrap_or_default();
        if ch != '&' {
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }

        let tail = &src[i + 1..];
        let mut semicolon_end = None;
        if let Some(semicolon_pos) = tail.find(';') {
            match tail.find('&') {
                Some(next_amp_pos) if next_amp_pos < semicolon_pos => {}
                _ => semicolon_end = Some(semicolon_pos),
            }
        }

        let Some(end_offset) = semicolon_end else {
            let entity_end = tail
                .char_indices()
                .find_map(|(idx, ch)| {
                    if is_entity_token_char(ch) {
                        None
                    } else {
                        Some(idx)
                    }
                })
                .unwrap_or(tail.len());

            if entity_end == 0 {
                out.push('&');
                i += 1;
                continue;
            }

            let raw = &tail[..entity_end];
            let decoded = if let Some(rest) = raw.strip_prefix('#') {
                decode_numeric(rest)
            } else {
                decode_named(raw)
            };

            if let Some(value) = decoded {
                out.push(value);
                i += entity_end + 1;
            } else {
                out.push('&');
                i += 1;
            }
            continue;
        };

        let raw = &tail[..end_offset];
        let decoded = if let Some(rest) = raw.strip_prefix('#') {
            decode_numeric(rest)
        } else {
            decode_named(raw)
        };

        if let Some(value) = decoded {
            out.push(value);
            i += end_offset + 2;
        } else {
            out.push('&');
            i += 1;
        }
    }

    out
}

pub(super) fn should_strip_initial_pre_newline(dom: &Dom, parent: NodeId) -> bool {
    dom.tag_name(parent)
        .is_some_and(|tag| tag.eq_ignore_ascii_case("pre"))
        && dom.nodes[parent.0].children.is_empty()
}

pub(super) fn strip_initial_pre_newline(text: &str) -> String {
    if let Some(rest) = text.strip_prefix("\r\n") {
        return rest.to_string();
    }
    if let Some(rest) = text.strip_prefix('\n') {
        return rest.to_string();
    }
    if let Some(rest) = text.strip_prefix('\r') {
        return rest.to_string();
    }
    text.to_string()
}
