use super::*;

pub(crate) fn parse_selector_step(part: &str) -> Result<SelectorStep> {
    let part = part.trim();
    if part.is_empty() {
        return Err(Error::UnsupportedSelector(part.into()));
    }

    let bytes = part.as_bytes();
    let mut i = 0usize;
    let mut step = SelectorStep::default();

    while i < bytes.len() {
        match bytes[i] {
            b'*' => {
                if step.universal {
                    return Err(Error::UnsupportedSelector(part.into()));
                }
                step.universal = true;
                i += 1;
            }
            b'#' => {
                i += 1;
                let Some((id, next)) = parse_selector_ident(part, i) else {
                    return Err(Error::UnsupportedSelector(part.into()));
                };
                if step.id.replace(id).is_some() {
                    return Err(Error::UnsupportedSelector(part.into()));
                }
                i = next;
            }
            b'.' => {
                i += 1;
                let Some((class_name, next)) = parse_selector_ident(part, i) else {
                    return Err(Error::UnsupportedSelector(part.into()));
                };
                step.classes.push(class_name);
                i = next;
            }
            b'[' => {
                let (attr, next) = parse_selector_attr_condition(part, i)?;
                step.attrs.push(attr);
                i = next;
            }
            b':' => {
                let Some((pseudo, next)) = parse_selector_pseudo(part, i) else {
                    return Err(Error::UnsupportedSelector(part.into()));
                };
                step.pseudo_classes.push(pseudo);
                i = next;
            }
            _ => {
                if step.tag.is_some()
                    || step.id.is_some()
                    || !step.classes.is_empty()
                    || step.universal
                {
                    return Err(Error::UnsupportedSelector(part.into()));
                }
                let Some((tag, next)) = parse_selector_ident(part, i) else {
                    return Err(Error::UnsupportedSelector(part.into()));
                };
                step.tag = Some(tag);
                i = next;
            }
        }
    }

    if step.tag.is_none()
        && step.id.is_none()
        && step.classes.is_empty()
        && step.attrs.is_empty()
        && !step.universal
        && step.pseudo_classes.is_empty()
    {
        return Err(Error::UnsupportedSelector(part.into()));
    }
    Ok(step)
}

pub(crate) fn parse_selector_pseudo(
    part: &str,
    start: usize,
) -> Option<(SelectorPseudoClass, usize)> {
    if part.as_bytes().get(start)? != &b':' {
        return None;
    }
    let start = start + 1;
    let tail = part.get(start..)?;
    if let Some(rest) = tail.strip_prefix("scope") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "scope".len();
            return Some((SelectorPseudoClass::Scope, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("first-child") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "first-child".len();
            return Some((SelectorPseudoClass::FirstChild, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("last-child") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "last-child".len();
            return Some((SelectorPseudoClass::LastChild, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("first-of-type") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "first-of-type".len();
            return Some((SelectorPseudoClass::FirstOfType, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("last-of-type") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "last-of-type".len();
            return Some((SelectorPseudoClass::LastOfType, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("only-child") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "only-child".len();
            return Some((SelectorPseudoClass::OnlyChild, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("only-of-type") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "only-of-type".len();
            return Some((SelectorPseudoClass::OnlyOfType, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("checked") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "checked".len();
            return Some((SelectorPseudoClass::Checked, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("indeterminate") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "indeterminate".len();
            return Some((SelectorPseudoClass::Indeterminate, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("disabled") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "disabled".len();
            return Some((SelectorPseudoClass::Disabled, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("required") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "required".len();
            return Some((SelectorPseudoClass::Required, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("optional") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "optional".len();
            return Some((SelectorPseudoClass::Optional, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("read-only") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "read-only".len();
            return Some((SelectorPseudoClass::Readonly, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("readonly") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "readonly".len();
            return Some((SelectorPseudoClass::Readonly, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("read-write") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "read-write".len();
            return Some((SelectorPseudoClass::Readwrite, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("empty") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "empty".len();
            return Some((SelectorPseudoClass::Empty, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("focus-within") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "focus-within".len();
            return Some((SelectorPseudoClass::FocusWithin, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("focus") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "focus".len();
            return Some((SelectorPseudoClass::Focus, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("active") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "active".len();
            return Some((SelectorPseudoClass::Active, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("enabled") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "enabled".len();
            return Some((SelectorPseudoClass::Enabled, consumed));
        }
    }

    if let Some((inners, next)) = parse_pseudo_selector_list(part, start, "not(") {
        return Some((SelectorPseudoClass::Not(inners), next));
    }

    if let Some((inners, next)) = parse_pseudo_selector_list(part, start, "is(") {
        return Some((SelectorPseudoClass::Is(inners), next));
    }

    if let Some((inners, next)) = parse_pseudo_selector_list(part, start, "where(") {
        return Some((SelectorPseudoClass::Where(inners), next));
    }

    if let Some((inners, next)) = parse_pseudo_selector_list(part, start, "has(") {
        return Some((SelectorPseudoClass::Has(inners), next));
    }

    if let Some(rest) = tail.strip_prefix("nth-last-of-type(") {
        let Some(close_pos) = find_matching_paren(rest) else {
            return None;
        };
        let raw = rest[..close_pos].trim();
        if raw.is_empty() {
            return None;
        }
        let selector = parse_nth_child_selector(raw)?;
        let next = start + "nth-last-of-type(".len() + close_pos + 1;
        if let Some(ch) = part.as_bytes().get(next) {
            if !is_selector_continuation(ch) {
                return None;
            }
        }
        return Some((SelectorPseudoClass::NthLastOfType(selector), next));
    }

    if let Some(rest) = tail.strip_prefix("nth-of-type(") {
        let Some(close_pos) = find_matching_paren(rest) else {
            return None;
        };
        let raw = rest[..close_pos].trim();
        if raw.is_empty() {
            return None;
        }
        let selector = parse_nth_child_selector(raw)?;
        let next = start + "nth-of-type(".len() + close_pos + 1;
        if let Some(ch) = part.as_bytes().get(next) {
            if !is_selector_continuation(ch) {
                return None;
            }
        }
        return Some((SelectorPseudoClass::NthOfType(selector), next));
    }

    if let Some(rest) = tail.strip_prefix("nth-last-child(") {
        let Some(close_pos) = find_matching_paren(rest) else {
            return None;
        };
        let raw = rest[..close_pos].trim();
        if raw.is_empty() {
            return None;
        }
        let selector = parse_nth_child_selector(raw)?;
        let next = start + "nth-last-child(".len() + close_pos + 1;
        if let Some(ch) = part.as_bytes().get(next) {
            if !is_selector_continuation(ch) {
                return None;
            }
        }
        return Some((SelectorPseudoClass::NthLastChild(selector), next));
    }

    if let Some(rest) = tail.strip_prefix("nth-child(") {
        let Some(close_pos) = find_matching_paren(rest) else {
            return None;
        };
        let raw = rest[..close_pos].trim();
        if raw.is_empty() {
            return None;
        }
        let selector = parse_nth_child_selector(raw)?;
        let next = start + "nth-child(".len() + close_pos + 1;
        if let Some(ch) = part.as_bytes().get(next) {
            if !is_selector_continuation(ch) {
                return None;
            }
        }
        return Some((SelectorPseudoClass::NthChild(selector), next));
    }

    None
}

pub(crate) fn parse_pseudo_selector_list(
    part: &str,
    start: usize,
    prefix: &str,
) -> Option<(Vec<Vec<SelectorPart>>, usize)> {
    let Some(rest) = part.get(start..).and_then(|tail| tail.strip_prefix(prefix)) else {
        return None;
    };

    let Some(close_pos) = find_matching_paren(rest) else {
        return None;
    };
    let body = rest[..close_pos].trim();
    if body.is_empty() {
        return None;
    }

    let mut groups = split_selector_groups(body).ok()?;
    if groups.is_empty() {
        return None;
    }

    let mut selectors = Vec::with_capacity(groups.len());
    for group in &mut groups {
        let chain = parse_selector_chain(group.trim()).ok()?;
        if chain.is_empty() {
            return None;
        }
        selectors.push(chain);
    }

    let next = start + prefix.len() + close_pos + 1;
    if let Some(ch) = part.as_bytes().get(next) {
        if !is_selector_continuation(ch) {
            return None;
        }
    }
    Some((selectors, next))
}

pub(crate) fn is_selector_continuation(next: &u8) -> bool {
    matches!(next, b'.' | b'#' | b'[' | b':')
}

pub(crate) fn parse_selector_ident(src: &str, start: usize) -> Option<(String, usize)> {
    let bytes = src.as_bytes();
    if start >= bytes.len() || !is_selector_ident_char(bytes[start]) {
        return None;
    }
    let mut end = start + 1;
    while end < bytes.len() && is_selector_ident_char(bytes[end]) {
        end += 1;
    }
    Some((src.get(start..end)?.to_string(), end))
}

pub(crate) fn is_selector_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}
