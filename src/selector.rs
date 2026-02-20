use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SelectorAttrCondition {
    Exists { key: String },
    Eq { key: String, value: String },
    StartsWith { key: String, value: String },
    EndsWith { key: String, value: String },
    Contains { key: String, value: String },
    Includes { key: String, value: String },
    DashMatch { key: String, value: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SelectorPseudoClass {
    FirstChild,
    LastChild,
    FirstOfType,
    LastOfType,
    OnlyChild,
    OnlyOfType,
    Checked,
    Indeterminate,
    Disabled,
    Enabled,
    Required,
    Optional,
    Readonly,
    Readwrite,
    Empty,
    Focus,
    FocusWithin,
    Active,
    NthOfType(NthChildSelector),
    NthLastOfType(NthChildSelector),
    Not(Vec<Vec<SelectorPart>>),
    Is(Vec<Vec<SelectorPart>>),
    Where(Vec<Vec<SelectorPart>>),
    Has(Vec<Vec<SelectorPart>>),
    NthChild(NthChildSelector),
    NthLastChild(NthChildSelector),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NthChildSelector {
    Exact(usize),
    Odd,
    Even,
    AnPlusB(i64, i64),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SelectorStep {
    pub(crate) tag: Option<String>,
    pub(crate) universal: bool,
    pub(crate) id: Option<String>,
    pub(crate) classes: Vec<String>,
    pub(crate) attrs: Vec<SelectorAttrCondition>,
    pub(crate) pseudo_classes: Vec<SelectorPseudoClass>,
}

impl SelectorStep {
    pub(crate) fn id_only(&self) -> Option<&str> {
        if !self.universal
            && self.tag.is_none()
            && self.classes.is_empty()
            && self.attrs.is_empty()
            && self.pseudo_classes.is_empty()
        {
            self.id.as_deref()
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectorCombinator {
    Descendant,
    Child,
    AdjacentSibling,
    GeneralSibling,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectorPart {
    pub(crate) step: SelectorStep,
    // Relation to previous (left) selector part.
    pub(crate) combinator: Option<SelectorCombinator>,
}

pub(crate) fn parse_selector_chain(selector: &str) -> Result<Vec<SelectorPart>> {
    let selector = selector.trim();
    if selector.is_empty() {
        return Err(Error::UnsupportedSelector(selector.into()));
    }

    let tokens = tokenize_selector(selector)?;
    let mut steps = Vec::new();
    let mut pending_combinator: Option<SelectorCombinator> = None;

    for token in tokens {
        if token == ">" || token == "+" || token == "~" {
            if pending_combinator.is_some() || steps.is_empty() {
                return Err(Error::UnsupportedSelector(selector.into()));
            }
            pending_combinator = Some(match token.as_str() {
                ">" => SelectorCombinator::Child,
                "+" => SelectorCombinator::AdjacentSibling,
                "~" => SelectorCombinator::GeneralSibling,
                _ => unreachable!(),
            });
            continue;
        }

        let step = parse_selector_step(&token)?;
        let combinator = if steps.is_empty() {
            None
        } else {
            Some(
                pending_combinator
                    .take()
                    .unwrap_or(SelectorCombinator::Descendant),
            )
        };
        steps.push(SelectorPart { step, combinator });
    }

    if steps.is_empty() || pending_combinator.is_some() {
        return Err(Error::UnsupportedSelector(selector.into()));
    }

    Ok(steps)
}

pub(crate) fn parse_selector_groups(selector: &str) -> Result<Vec<Vec<SelectorPart>>> {
    let groups = split_selector_groups(selector)?;
    let mut parsed = Vec::with_capacity(groups.len());
    for group in groups {
        parsed.push(parse_selector_chain(&group)?);
    }
    Ok(parsed)
}

pub(crate) fn split_selector_groups(selector: &str) -> Result<Vec<String>> {
    let mut groups = Vec::new();
    let mut current = String::new();
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;

    for ch in selector.chars() {
        match ch {
            '[' => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' => {
                if bracket_depth == 0 {
                    return Err(Error::UnsupportedSelector(selector.into()));
                }
                bracket_depth -= 1;
                current.push(ch);
            }
            '(' => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' => {
                if paren_depth == 0 {
                    return Err(Error::UnsupportedSelector(selector.into()));
                }
                paren_depth -= 1;
                current.push(ch);
            }
            ',' if bracket_depth == 0 && paren_depth == 0 => {
                let trimmed = current.trim();
                if trimmed.is_empty() {
                    return Err(Error::UnsupportedSelector(selector.into()));
                }
                groups.push(trimmed.to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if bracket_depth != 0 {
        return Err(Error::UnsupportedSelector(selector.into()));
    }
    if paren_depth != 0 {
        return Err(Error::UnsupportedSelector(selector.into()));
    }

    let trimmed = current.trim();
    if trimmed.is_empty() {
        return Err(Error::UnsupportedSelector(selector.into()));
    }
    groups.push(trimmed.to_string());
    Ok(groups)
}

pub(crate) fn tokenize_selector(selector: &str) -> Result<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;

    for ch in selector.chars() {
        match ch {
            '[' => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' => {
                if bracket_depth == 0 {
                    return Err(Error::UnsupportedSelector(selector.into()));
                }
                bracket_depth -= 1;
                current.push(ch);
            }
            '(' => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' => {
                if paren_depth == 0 {
                    return Err(Error::UnsupportedSelector(selector.into()));
                }
                paren_depth -= 1;
                current.push(ch);
            }
            '>' | '+' | '~' if bracket_depth == 0 && paren_depth == 0 => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                }
                current.clear();
                tokens.push(ch.to_string());
            }
            ch if ch.is_ascii_whitespace() && bracket_depth == 0 && paren_depth == 0 => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if bracket_depth != 0 {
        return Err(Error::UnsupportedSelector(selector.into()));
    }
    if paren_depth != 0 {
        return Err(Error::UnsupportedSelector(selector.into()));
    }

    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }

    Ok(tokens)
}

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
        let body = rest;
        let Some(close_pos) = find_matching_paren(body) else {
            return None;
        };
        let raw = body[..close_pos].trim();
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
        let body = rest;
        let Some(close_pos) = find_matching_paren(body) else {
            return None;
        };
        let raw = body[..close_pos].trim();
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
        let body = rest;
        let Some(close_pos) = find_matching_paren(body) else {
            return None;
        };
        let raw = body[..close_pos].trim();
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
        let body = rest;
        let Some(close_pos) = find_matching_paren(body) else {
            return None;
        };
        let raw = body[..close_pos].trim();
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
            } else {
                if other.starts_with('+') || other.starts_with('-') {
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
