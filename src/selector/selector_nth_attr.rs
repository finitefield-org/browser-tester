use super::*;

pub(crate) fn find_matching_paren(body: &str) -> Option<usize> {
    let mut paren_depth = 1usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<u8> = None;
    let mut escaped = false;

    for (idx, b) in body.bytes().enumerate() {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if b == b'\\' {
                escaped = true;
                continue;
            }
            if b == q {
                quote = None;
            }
            continue;
        }

        match b {
            b'\'' | b'"' => quote = Some(b),
            b'[' => {
                bracket_depth += 1;
            }
            b']' => {
                if bracket_depth == 0 {
                    return None;
                }
                bracket_depth -= 1;
            }
            b'(' if bracket_depth == 0 => {
                paren_depth += 1;
            }
            b')' if bracket_depth == 0 => {
                paren_depth = paren_depth.checked_sub(1)?;
                if paren_depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

pub(crate) fn parse_nth_child_selector(raw: &str) -> Option<NthChildSelector> {
    let compact = raw
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    if compact.is_empty() {
        return None;
    }

    match compact.as_str() {
        "odd" => Some(NthChildSelector::Odd),
        "even" => Some(NthChildSelector::Even),
        other => {
            if other.contains('n') {
                parse_nth_child_expression(other)
            } else if other.starts_with('+') || other.starts_with('-') {
                None
            } else {
                let value = other.parse::<usize>().ok()?;
                if value == 0 {
                    None
                } else {
                    Some(NthChildSelector::Exact(value))
                }
            }
        }
    }
}

pub(crate) fn parse_nth_child_expression(raw: &str) -> Option<NthChildSelector> {
    let expr = raw
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect::<String>();
    let expr = expr.to_ascii_lowercase();
    if expr.matches('n').count() != 1 {
        return None;
    }
    if expr.starts_with(|c: char| c == '+' || c == '-') && expr.len() == 1 {
        return None;
    }

    let n_pos = expr.find('n')?;
    let (a_part, rest) = expr.split_at(n_pos);
    let b_part = &rest[1..];

    let a = match a_part {
        "" => 1,
        "-" => -1,
        "+" => return None,
        _ => a_part.parse::<i64>().ok()?,
    };

    if b_part.is_empty() {
        return Some(NthChildSelector::AnPlusB(a, 0));
    }

    let mut sign = 1;
    let raw_b = if let Some(rest) = b_part.strip_prefix('+') {
        rest
    } else if let Some(rest) = b_part.strip_prefix('-') {
        sign = -1;
        rest
    } else {
        return None;
    };
    if raw_b.is_empty() {
        return None;
    }
    let b = raw_b.parse::<i64>().ok()?;
    Some(NthChildSelector::AnPlusB(a, b * sign))
}

pub(crate) fn parse_selector_attr_condition(
    src: &str,
    open_bracket: usize,
) -> Result<(SelectorAttrCondition, usize)> {
    let bytes = src.as_bytes();
    let mut i = open_bracket + 1;

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() {
        return Err(Error::UnsupportedSelector(src.into()));
    }

    let key_start = i;
    while i < bytes.len() {
        if is_selector_attr_name_char(bytes[i]) {
            i += 1;
            continue;
        }
        break;
    }
    if key_start == i {
        return Err(Error::UnsupportedSelector(src.into()));
    }
    let key = src
        .get(key_start..i)
        .ok_or_else(|| Error::UnsupportedSelector(src.into()))?
        .to_ascii_lowercase();

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() {
        return Err(Error::UnsupportedSelector(src.into()));
    }

    if bytes[i] == b']' {
        return Ok((SelectorAttrCondition::Exists { key }, i + 1));
    }

    let (op, mut next) = match bytes.get(i) {
        Some(b'=') => (SelectorAttrConditionType::Eq, i + 1),
        Some(b'^') if bytes.get(i + 1) == Some(&b'=') => {
            (SelectorAttrConditionType::StartsWith, i + 2)
        }
        Some(b'$') if bytes.get(i + 1) == Some(&b'=') => {
            (SelectorAttrConditionType::EndsWith, i + 2)
        }
        Some(b'*') if bytes.get(i + 1) == Some(&b'=') => {
            (SelectorAttrConditionType::Contains, i + 2)
        }
        Some(b'~') if bytes.get(i + 1) == Some(&b'=') => {
            (SelectorAttrConditionType::Includes, i + 2)
        }
        Some(b'|') if bytes.get(i + 1) == Some(&b'=') => {
            (SelectorAttrConditionType::DashMatch, i + 2)
        }
        _ => return Err(Error::UnsupportedSelector(src.into())),
    };

    i = next;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() {
        return Err(Error::UnsupportedSelector(src.into()));
    }

    let (value, after_value) = parse_selector_attr_value(src, i)?;
    next = after_value;

    i = next;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b']' {
        return Err(Error::UnsupportedSelector(src.into()));
    }

    let cond = match op {
        SelectorAttrConditionType::Eq => SelectorAttrCondition::Eq { key, value },
        SelectorAttrConditionType::StartsWith => SelectorAttrCondition::StartsWith { key, value },
        SelectorAttrConditionType::EndsWith => SelectorAttrCondition::EndsWith { key, value },
        SelectorAttrConditionType::Contains => SelectorAttrCondition::Contains { key, value },
        SelectorAttrConditionType::Includes => SelectorAttrCondition::Includes { key, value },
        SelectorAttrConditionType::DashMatch => SelectorAttrCondition::DashMatch { key, value },
    };

    Ok((cond, i + 1))
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SelectorAttrConditionType {
    Eq,
    StartsWith,
    EndsWith,
    Contains,
    Includes,
    DashMatch,
}

pub(crate) fn is_selector_attr_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b':'
}

pub(crate) fn parse_selector_attr_value(src: &str, start: usize) -> Result<(String, usize)> {
    let bytes = src.as_bytes();
    if start >= bytes.len() {
        return Err(Error::UnsupportedSelector(src.into()));
    }

    if bytes[start] == b'"' || bytes[start] == b'\'' {
        let quote = bytes[start];
        let mut i = start + 1;
        while i < bytes.len() {
            if bytes[i] == b'\\' {
                i = (i + 2).min(bytes.len());
                continue;
            }
            if bytes[i] == quote {
                let raw = src
                    .get(start + 1..i)
                    .ok_or_else(|| Error::UnsupportedSelector(src.into()))?;
                return Ok((unescape_string(raw), i + 1));
            }
            i += 1;
        }
        return Err(Error::UnsupportedSelector(src.into()));
    }

    let start_value = start;
    let mut i = start;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() || bytes[i] == b']' {
            break;
        }
        if bytes[i] == b'\\' {
            i = (i + 2).min(bytes.len());
            continue;
        }
        i += 1;
    }
    if i == start_value {
        return Ok(("".to_string(), i));
    }
    let raw = src
        .get(start_value..i)
        .ok_or_else(|| Error::UnsupportedSelector(src.into()))?;
    Ok((unescape_string(raw), i))
}
