pub(crate) fn validate_xml_well_formed(markup: &str) -> std::result::Result<(), String> {
    let bytes = markup.as_bytes();
    let mut index = 0usize;
    let mut stack = Vec::<String>::new();
    let mut root_seen = false;

    while index < bytes.len() {
        if bytes[index] != b'<' {
            if stack.is_empty() && !bytes[index].is_ascii_whitespace() {
                return Err("XML text is not allowed outside the root element".into());
            }
            index += 1;
            continue;
        }

        if xml_starts_with_at(bytes, index, b"<!--") {
            index = find_xml_subslice(bytes, index + 4, b"-->")
                .map(|end| end + 3)
                .ok_or_else(|| "Unclosed XML comment".to_string())?;
            continue;
        }

        if xml_starts_with_at(bytes, index, b"<![CDATA[") {
            index = find_xml_subslice(bytes, index + 9, b"]]>")
                .map(|end| end + 3)
                .ok_or_else(|| "Unclosed CDATA section".to_string())?;
            continue;
        }

        if xml_starts_with_at(bytes, index, b"<?") {
            index = find_xml_subslice(bytes, index + 2, b"?>")
                .map(|end| end + 2)
                .ok_or_else(|| "Unclosed XML processing instruction".to_string())?;
            continue;
        }

        if xml_starts_with_at(bytes, index, b"<!") {
            index = skip_xml_declaration(bytes, index)?;
            continue;
        }

        if xml_starts_with_at(bytes, index, b"</") {
            index += 2;
            skip_xml_whitespace(bytes, &mut index);
            let name = parse_xml_name(markup, bytes, &mut index)?;
            skip_xml_whitespace(bytes, &mut index);
            if bytes.get(index) != Some(&b'>') {
                return Err("Malformed XML closing tag".into());
            }
            index += 1;

            let Some(open_name) = stack.pop() else {
                return Err(format!("Unexpected closing tag </{name}>"));
            };
            if open_name != name {
                return Err(format!(
                    "Mismatched XML closing tag </{name}> for <{open_name}>"
                ));
            }
            continue;
        }

        index += 1;
        skip_xml_whitespace(bytes, &mut index);
        let name = parse_xml_name(markup, bytes, &mut index)?;
        if stack.is_empty() {
            if root_seen {
                return Err("XML documents must have a single root element".into());
            }
            root_seen = true;
        }
        let self_closing = skip_xml_start_tag_tail(markup, bytes, &mut index)?;
        if !self_closing {
            stack.push(name);
        }
    }

    if !stack.is_empty() {
        return Err("Unclosed XML element".into());
    }
    if !root_seen {
        return Err("XML document is missing a root element".into());
    }
    Ok(())
}

fn skip_xml_start_tag_tail(
    markup: &str,
    bytes: &[u8],
    index: &mut usize,
) -> std::result::Result<bool, String> {
    loop {
        skip_xml_whitespace(bytes, index);
        match bytes.get(*index) {
            Some(b'>') => {
                *index += 1;
                return Ok(false);
            }
            Some(b'/') if bytes.get(*index + 1) == Some(&b'>') => {
                *index += 2;
                return Ok(true);
            }
            Some(_) => {
                let _ = parse_xml_name(markup, bytes, index)?;
                skip_xml_whitespace(bytes, index);
                if bytes.get(*index) != Some(&b'=') {
                    return Err("XML attributes must have explicit values".into());
                }
                *index += 1;
                skip_xml_whitespace(bytes, index);
                let Some(quote) = bytes.get(*index).copied() else {
                    return Err("Unclosed XML attribute value".into());
                };
                if !matches!(quote, b'\'' | b'"') {
                    return Err("XML attribute values must be quoted".into());
                }
                *index += 1;
                while let Some(&byte) = bytes.get(*index) {
                    if byte == quote {
                        *index += 1;
                        break;
                    }
                    *index += 1;
                }
                if bytes.get((*index).saturating_sub(1)) != Some(&quote) {
                    return Err("Unclosed XML attribute value".into());
                }
            }
            None => return Err("Unclosed XML start tag".into()),
        }
    }
}

fn skip_xml_declaration(bytes: &[u8], start: usize) -> std::result::Result<usize, String> {
    let mut index = start + 2;
    let mut single_quoted = false;
    let mut double_quoted = false;
    let mut bracket_depth = 0usize;

    while let Some(&byte) = bytes.get(index) {
        if single_quoted {
            if byte == b'\'' {
                single_quoted = false;
            }
            index += 1;
            continue;
        }

        if double_quoted {
            if byte == b'"' {
                double_quoted = false;
            }
            index += 1;
            continue;
        }

        match byte {
            b'\'' => single_quoted = true,
            b'"' => double_quoted = true,
            b'[' => bracket_depth += 1,
            b']' if bracket_depth > 0 => bracket_depth -= 1,
            b'>' if bracket_depth == 0 => return Ok(index + 1),
            _ => {}
        }

        index += 1;
    }

    Err("Unclosed XML declaration".into())
}

fn parse_xml_name(
    markup: &str,
    bytes: &[u8],
    index: &mut usize,
) -> std::result::Result<String, String> {
    let start = *index;
    let Some(&first) = bytes.get(*index) else {
        return Err("Missing XML name".into());
    };
    if !is_xml_name_start_char(first) {
        return Err("Invalid XML name".into());
    }
    *index += 1;
    while let Some(&byte) = bytes.get(*index) {
        if !is_xml_name_char(byte) {
            break;
        }
        *index += 1;
    }
    markup
        .get(start..*index)
        .map(ToString::to_string)
        .ok_or_else(|| "Invalid XML name".to_string())
}

fn skip_xml_whitespace(bytes: &[u8], index: &mut usize) {
    while bytes
        .get(*index)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        *index += 1;
    }
}

fn is_xml_name_start_char(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b':')
}

fn is_xml_name_char(byte: u8) -> bool {
    is_xml_name_start_char(byte) || byte.is_ascii_digit() || matches!(byte, b'-' | b'.')
}

fn xml_starts_with_at(bytes: &[u8], index: usize, needle: &[u8]) -> bool {
    bytes
        .get(index..index + needle.len())
        .is_some_and(|slice| slice == needle)
}

fn find_xml_subslice(bytes: &[u8], start: usize, needle: &[u8]) -> Option<usize> {
    bytes
        .get(start..)?
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| start + offset)
}
