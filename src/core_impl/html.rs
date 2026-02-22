use super::*;

pub(super) fn unescape_string(src: &str) -> String {
    let mut out = String::new();
    let chars = src.chars().collect::<Vec<_>>();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'n' => {
                    out.push('\n');
                    i += 2;
                }
                'r' => {
                    out.push('\r');
                    i += 2;
                }
                't' => {
                    out.push('\t');
                    i += 2;
                }
                '\\' => {
                    out.push('\\');
                    i += 2;
                }
                '\'' => {
                    out.push('\'');
                    i += 2;
                }
                '"' => {
                    out.push('"');
                    i += 2;
                }
                '`' => {
                    out.push('`');
                    i += 2;
                }
                '$' => {
                    out.push('$');
                    i += 2;
                }
                'u' if i + 5 < chars.len() => {
                    let hex = [chars[i + 2], chars[i + 3], chars[i + 4], chars[i + 5]];
                    let mut parsed = String::new();
                    for ch in hex {
                        parsed.push(ch);
                    }
                    if parsed.chars().all(|ch| ch.is_ascii_hexdigit()) {
                        if let Ok(codepoint) = u16::from_str_radix(&parsed, 16) {
                            out.push(crate::js_regex::internalize_utf16_code_unit(codepoint));
                            i += 6;
                            continue;
                        }
                    }
                    out.push('u');
                    i += 2;
                }
                'x' if i + 3 < chars.len() => {
                    let hex = [chars[i + 2], chars[i + 3]];
                    let mut parsed = String::new();
                    for ch in hex {
                        parsed.push(ch);
                    }
                    if parsed.chars().all(|ch| ch.is_ascii_hexdigit()) {
                        if let Ok(codepoint) = u32::from_str_radix(&parsed, 16) {
                            if let Some(ch) = char::from_u32(codepoint) {
                                out.push(ch);
                                i += 4;
                                continue;
                            }
                        }
                    }
                    out.push('x');
                    i += 2;
                }
                other => {
                    out.push(other);
                    i += 2;
                }
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn decode_html_character_references(src: &str) -> String {
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

pub(super) fn parse_html(html: &str) -> Result<ParseOutput> {
    let mut dom = Dom::new();
    let mut scripts = Vec::new();

    let mut stack = vec![dom.root];
    let bytes = html.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        if starts_with_at(bytes, i, b"<!--") {
            if let Some(end) = find_subslice(bytes, i + 4, b"-->") {
                i = end + 3;
            } else {
                return Err(Error::HtmlParse("unclosed HTML comment".into()));
            }
            continue;
        }

        if bytes[i] == b'<' {
            if starts_with_at(bytes, i, b"</") {
                let (tag, next) = parse_end_tag(html, i)?;
                i = next;

                while stack.len() > 1 {
                    let top = *stack
                        .last()
                        .ok_or_else(|| Error::HtmlParse("invalid stack state".into()))?;
                    let top_tag = dom.tag_name(top).unwrap_or("");
                    stack.pop();
                    if top_tag.eq_ignore_ascii_case(&tag) {
                        break;
                    }
                }
                continue;
            }

            if starts_with_at(bytes, i, b"<!") {
                i = parse_declaration_tag(html, i)?;
                continue;
            }

            let (tag, attrs, self_closing, next) = parse_start_tag(html, i)?;
            i = next;
            let inside_template = stack.iter().any(|node| {
                dom.tag_name(*node)
                    .is_some_and(|open_tag| open_tag.eq_ignore_ascii_case("template"))
            });
            let executable_script = tag.eq_ignore_ascii_case("script")
                && !inside_template
                && is_executable_script_type(attrs.get("type").map(String::as_str));
            close_optional_description_item_start_tag(&dom, &mut stack, &tag);
            close_optional_list_item_start_tag(&dom, &mut stack, &tag);
            close_optional_option_start_tag(&dom, &mut stack, &tag);
            close_optional_optgroup_start_tag(&dom, &mut stack, &tag);
            close_optional_ruby_text_start_tag(&dom, &mut stack, &tag);
            close_optional_ruby_fallback_parenthesis_start_tag(&dom, &mut stack, &tag);
            close_optional_paragraph_start_tag(&dom, &mut stack, &tag);

            let parent = *stack
                .last()
                .ok_or_else(|| Error::HtmlParse("missing parent element".into()))?;
            let node = dom.create_element(parent, tag.clone(), attrs);

            if tag.eq_ignore_ascii_case("script") {
                let close = if executable_script {
                    find_case_insensitive_end_tag(bytes, i, b"script")
                        // Some generated pages can contain malformed JS source. If the
                        // JS-aware scan cannot recover, fall back to raw end-tag search
                        // so HTML parsing still advances to the explicit </script>.
                        .or_else(|| find_case_insensitive_raw_end_tag(bytes, i, b"script"))
                } else {
                    find_case_insensitive_raw_end_tag(bytes, i, b"script")
                }
                .ok_or_else(|| Error::HtmlParse("unclosed <script>".into()))?;
                if let Some(script_body) = html.get(i..close) {
                    if !script_body.is_empty() {
                        dom.create_text(node, script_body.to_string());
                        if executable_script {
                            scripts.push(script_body.to_string());
                        }
                    }
                }
                i = close;
                let (_, after_end) = parse_end_tag(html, i)?;
                i = after_end;
                continue;
            }

            if tag.eq_ignore_ascii_case("noscript") && !self_closing {
                let close = find_case_insensitive_raw_end_tag(bytes, i, b"noscript")
                    .ok_or_else(|| Error::HtmlParse("unclosed <noscript>".into()))?;
                if let Some(noscript_body) = html.get(i..close) {
                    if !noscript_body.is_empty() {
                        dom.create_text(node, noscript_body.to_string());
                    }
                }
                i = close;
                let (_, after_end) = parse_end_tag(html, i)?;
                i = after_end;
                continue;
            }

            if tag.eq_ignore_ascii_case("title") && !self_closing {
                let close = find_case_insensitive_raw_end_tag(bytes, i, b"title")
                    .ok_or_else(|| Error::HtmlParse("unclosed <title>".into()))?;
                if let Some(title_body) = html.get(i..close) {
                    if !title_body.is_empty() {
                        let decoded = decode_html_character_references(title_body);
                        if !decoded.is_empty() {
                            dom.create_text(node, decoded);
                        }
                    }
                }
                i = close;
                let (_, after_end) = parse_end_tag(html, i)?;
                i = after_end;
                continue;
            }

            if !self_closing && !is_void_tag(&tag) {
                stack.push(node);
            }
            continue;
        }

        let text_start = i;
        while i < bytes.len() && bytes[i] != b'<' {
            i += 1;
        }

        if let Some(text) = html.get(text_start..i) {
            if !text.is_empty() {
                let parent = *stack
                    .last()
                    .ok_or_else(|| Error::HtmlParse("missing parent element".into()))?;
                let mut decoded = decode_html_character_references(text);
                if should_strip_initial_pre_newline(&dom, parent) {
                    decoded = strip_initial_pre_newline(&decoded);
                }
                if !decoded.is_empty() {
                    dom.create_text(parent, decoded);
                }
            }
        }
    }

    dom.initialize_form_control_values()?;
    dom.normalize_radio_groups()?;
    dom.normalize_named_details_groups()?;
    dom.normalize_single_head_element()?;
    dom.normalize_single_body_element()?;
    dom.normalize_implied_table_bodies()?;
    Ok(ParseOutput { dom, scripts })
}

fn close_optional_description_item_start_tag(dom: &Dom, stack: &mut Vec<NodeId>, tag: &str) {
    if !(tag.eq_ignore_ascii_case("dt") || tag.eq_ignore_ascii_case("dd")) {
        return;
    }

    let mut close_index = None;
    for index in (1..stack.len()).rev() {
        let Some(open_tag) = dom.tag_name(stack[index]) else {
            continue;
        };
        if open_tag.eq_ignore_ascii_case("dt") || open_tag.eq_ignore_ascii_case("dd") {
            close_index = Some(index);
            break;
        }
        if open_tag.eq_ignore_ascii_case("dl") {
            break;
        }
    }

    if let Some(index) = close_index {
        stack.truncate(index);
    }
}

fn close_optional_list_item_start_tag(dom: &Dom, stack: &mut Vec<NodeId>, tag: &str) {
    if !tag.eq_ignore_ascii_case("li") {
        return;
    }

    let mut close_index = None;
    for index in (1..stack.len()).rev() {
        let Some(open_tag) = dom.tag_name(stack[index]) else {
            continue;
        };
        if open_tag.eq_ignore_ascii_case("li") {
            close_index = Some(index);
            break;
        }
        if open_tag.eq_ignore_ascii_case("ol")
            || open_tag.eq_ignore_ascii_case("ul")
            || open_tag.eq_ignore_ascii_case("menu")
        {
            break;
        }
    }

    if let Some(index) = close_index {
        stack.truncate(index);
    }
}

fn close_optional_option_start_tag(dom: &Dom, stack: &mut Vec<NodeId>, tag: &str) {
    if !(tag.eq_ignore_ascii_case("option") || tag.eq_ignore_ascii_case("optgroup")) {
        return;
    }

    let mut close_index = None;
    for index in (1..stack.len()).rev() {
        let Some(open_tag) = dom.tag_name(stack[index]) else {
            continue;
        };
        if open_tag.eq_ignore_ascii_case("option") {
            close_index = Some(index);
            break;
        }
        if open_tag.eq_ignore_ascii_case("optgroup")
            || open_tag.eq_ignore_ascii_case("select")
            || open_tag.eq_ignore_ascii_case("datalist")
        {
            break;
        }
    }

    if let Some(index) = close_index {
        stack.truncate(index);
    }
}

fn close_optional_optgroup_start_tag(dom: &Dom, stack: &mut Vec<NodeId>, tag: &str) {
    if !tag.eq_ignore_ascii_case("optgroup") {
        return;
    }

    let mut close_index = None;
    for index in (1..stack.len()).rev() {
        let Some(open_tag) = dom.tag_name(stack[index]) else {
            continue;
        };
        if open_tag.eq_ignore_ascii_case("optgroup") {
            close_index = Some(index);
            break;
        }
        if open_tag.eq_ignore_ascii_case("select") {
            break;
        }
    }

    if let Some(index) = close_index {
        stack.truncate(index);
    }
}

fn close_optional_ruby_text_start_tag(dom: &Dom, stack: &mut Vec<NodeId>, tag: &str) {
    if !(tag.eq_ignore_ascii_case("rt") || tag.eq_ignore_ascii_case("rp")) {
        return;
    }

    let mut close_index = None;
    for index in (1..stack.len()).rev() {
        let Some(open_tag) = dom.tag_name(stack[index]) else {
            continue;
        };
        if open_tag.eq_ignore_ascii_case("rt") {
            close_index = Some(index);
            break;
        }
        if open_tag.eq_ignore_ascii_case("ruby") {
            break;
        }
    }

    if let Some(index) = close_index {
        stack.truncate(index);
    }
}

fn close_optional_ruby_fallback_parenthesis_start_tag(
    dom: &Dom,
    stack: &mut Vec<NodeId>,
    tag: &str,
) {
    if !(tag.eq_ignore_ascii_case("rt") || tag.eq_ignore_ascii_case("rp")) {
        return;
    }

    let mut close_index = None;
    for index in (1..stack.len()).rev() {
        let Some(open_tag) = dom.tag_name(stack[index]) else {
            continue;
        };
        if open_tag.eq_ignore_ascii_case("rp") {
            close_index = Some(index);
            break;
        }
        if open_tag.eq_ignore_ascii_case("ruby") {
            break;
        }
    }

    if let Some(index) = close_index {
        stack.truncate(index);
    }
}

fn close_optional_paragraph_start_tag(dom: &Dom, stack: &mut Vec<NodeId>, tag: &str) {
    if !is_optional_paragraph_terminator_tag(tag) {
        return;
    }

    let mut close_index = None;
    for index in (1..stack.len()).rev() {
        let Some(open_tag) = dom.tag_name(stack[index]) else {
            continue;
        };
        if open_tag.eq_ignore_ascii_case("p") {
            close_index = Some(index);
            break;
        }
    }

    if let Some(index) = close_index {
        stack.truncate(index);
    }
}

fn is_optional_paragraph_terminator_tag(tag: &str) -> bool {
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "details"
            | "div"
            | "dl"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hgroup"
            | "hr"
            | "main"
            | "menu"
            | "nav"
            | "ol"
            | "p"
            | "pre"
            | "search"
            | "section"
            | "table"
            | "ul"
    )
}

fn should_strip_initial_pre_newline(dom: &Dom, parent: NodeId) -> bool {
    dom.tag_name(parent)
        .is_some_and(|tag| tag.eq_ignore_ascii_case("pre"))
        && dom.nodes[parent.0].children.is_empty()
}

fn strip_initial_pre_newline(text: &str) -> String {
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

fn is_executable_script_type(raw_type: Option<&str>) -> bool {
    let Some(raw_type) = raw_type else {
        return true;
    };

    let media_type = raw_type
        .split(';')
        .next()
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase();

    if media_type.is_empty() {
        return true;
    }

    matches!(
        media_type.as_str(),
        "text/javascript"
            | "application/javascript"
            | "application/ecmascript"
            | "text/ecmascript"
            | "module"
    )
}

fn parse_start_tag(
    html: &str,
    at: usize,
) -> Result<(String, HashMap<String, String>, bool, usize)> {
    let bytes = html.as_bytes();
    let mut i = at;
    if bytes.get(i) != Some(&b'<') {
        return Err(Error::HtmlParse("expected '<'".into()));
    }
    i += 1;

    skip_ws(bytes, &mut i);
    let tag_start = i;
    while i < bytes.len() && is_tag_char(bytes[i]) {
        i += 1;
    }

    let tag = html
        .get(tag_start..i)
        .ok_or_else(|| Error::HtmlParse("invalid tag name".into()))?
        .to_ascii_lowercase();

    if tag.is_empty() {
        return Err(Error::HtmlParse("empty tag name".into()));
    }

    let mut attrs = HashMap::new();
    let mut self_closing = false;

    loop {
        skip_ws(bytes, &mut i);
        if i >= bytes.len() {
            return Err(Error::HtmlParse("unclosed start tag".into()));
        }

        if bytes[i] == b'>' {
            i += 1;
            break;
        }

        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'>' {
            self_closing = true;
            i += 2;
            break;
        }

        if !is_attr_name_char(bytes[i]) {
            // Browser engines recover from malformed attribute fragments
            // (e.g. href=""/en/"tools/") by skipping junk tokens.
            while i < bytes.len()
                && !bytes[i].is_ascii_whitespace()
                && bytes[i] != b'>'
                && !(bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'>')
            {
                i += 1;
            }
            continue;
        }

        let name_start = i;
        while i < bytes.len() && is_attr_name_char(bytes[i]) {
            i += 1;
        }

        let name = html
            .get(name_start..i)
            .ok_or_else(|| Error::HtmlParse("invalid attribute name".into()))?
            .to_ascii_lowercase();

        if name.is_empty() {
            return Err(Error::HtmlParse("invalid attribute name".into()));
        }

        skip_ws(bytes, &mut i);

        let value = if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            skip_ws(bytes, &mut i);
            parse_attr_value(html, bytes, &mut i)?
        } else {
            "true".to_string()
        };

        attrs.insert(name, value);
    }

    Ok((tag, attrs, self_closing, i))
}

fn parse_declaration_tag(html: &str, at: usize) -> Result<usize> {
    let bytes = html.as_bytes();
    let mut i = at;

    if !(bytes.get(i) == Some(&b'<') && bytes.get(i + 1) == Some(&b'!')) {
        return Err(Error::HtmlParse("expected declaration tag".into()));
    }
    i += 2;

    let mut single_quoted = false;
    let mut double_quoted = false;
    let mut bracket_depth = 0usize;

    while i < bytes.len() {
        let b = bytes[i];

        if single_quoted {
            if b == b'\'' {
                single_quoted = false;
            }
            i += 1;
            continue;
        }

        if double_quoted {
            if b == b'"' {
                double_quoted = false;
            }
            i += 1;
            continue;
        }

        match b {
            b'\'' => single_quoted = true,
            b'"' => double_quoted = true,
            b'[' => bracket_depth += 1,
            b']' if bracket_depth > 0 => bracket_depth -= 1,
            b'>' if bracket_depth == 0 => return Ok(i + 1),
            _ => {}
        }

        i += 1;
    }

    Err(Error::HtmlParse("unclosed declaration tag".into()))
}

fn parse_end_tag(html: &str, at: usize) -> Result<(String, usize)> {
    let bytes = html.as_bytes();
    let mut i = at;

    if !(bytes.get(i) == Some(&b'<') && bytes.get(i + 1) == Some(&b'/')) {
        return Err(Error::HtmlParse("expected end tag".into()));
    }
    i += 2;
    skip_ws(bytes, &mut i);

    let tag_start = i;
    while i < bytes.len() && is_tag_char(bytes[i]) {
        i += 1;
    }

    let tag = html
        .get(tag_start..i)
        .ok_or_else(|| Error::HtmlParse("invalid end tag".into()))?
        .to_ascii_lowercase();

    while i < bytes.len() && bytes[i] != b'>' {
        i += 1;
    }
    if i >= bytes.len() {
        return Err(Error::HtmlParse("unclosed end tag".into()));
    }

    Ok((tag, i + 1))
}

fn parse_attr_value(html: &str, bytes: &[u8], i: &mut usize) -> Result<String> {
    if *i >= bytes.len() {
        return Err(Error::HtmlParse("missing attribute value".into()));
    }

    if bytes[*i] == b'\'' || bytes[*i] == b'"' {
        let quote = bytes[*i];
        *i += 1;
        let start = *i;
        while *i < bytes.len() && bytes[*i] != quote {
            *i += 1;
        }
        if *i >= bytes.len() {
            return Err(Error::HtmlParse("unclosed quoted attribute value".into()));
        }
        let value = html
            .get(start..*i)
            .ok_or_else(|| Error::HtmlParse("invalid attribute value".into()))?
            .to_string();
        *i += 1;
        return Ok(decode_html_character_references(&value));
    }

    let start = *i;
    while *i < bytes.len()
        && !bytes[*i].is_ascii_whitespace()
        && bytes[*i] != b'>'
        && !(bytes[*i] == b'/' && *i + 1 < bytes.len() && bytes[*i + 1] == b'>')
    {
        *i += 1;
    }

    let value = html
        .get(start..*i)
        .ok_or_else(|| Error::HtmlParse("invalid attribute value".into()))?
        .to_string();
    Ok(decode_html_character_references(&value))
}

fn skip_ws(bytes: &[u8], i: &mut usize) {
    while *i < bytes.len() && bytes[*i].is_ascii_whitespace() {
        *i += 1;
    }
}

fn is_tag_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

fn is_attr_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b':'
}

pub(super) fn is_void_tag(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn starts_with_at(bytes: &[u8], at: usize, needle: &[u8]) -> bool {
    if at + needle.len() > bytes.len() {
        return false;
    }
    &bytes[at..at + needle.len()] == needle
}

fn find_subslice(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || from > bytes.len() {
        return None;
    }

    let mut i = from;
    while i + needle.len() <= bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

pub(super) fn can_start_regex_literal(previous: Option<u8>) -> bool {
    match previous {
        None => true,
        Some(byte) => matches!(
            byte,
            b'(' | b'['
                | b'{'
                | b','
                | b';'
                | b':'
                | b'='
                | b'!'
                | b'?'
                | b'&'
                | b'|'
                | b'^'
                | b'~'
                | b'<'
                | b'>'
                | b'+'
                | b'-'
                | b'*'
                | b'%'
                | b'/'
        ),
    }
}

fn find_case_insensitive_end_tag(bytes: &[u8], from: usize, tag: &[u8]) -> Option<usize> {
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

fn find_case_insensitive_raw_end_tag(bytes: &[u8], from: usize, tag: &[u8]) -> Option<usize> {
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
