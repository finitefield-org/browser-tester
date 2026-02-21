use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub(crate) struct Regex {
    ast: Expr,
    capture_count: usize,
    named_captures: HashMap<String, usize>,
    named_capture_order: Vec<String>,
    ignore_case: bool,
    multi_line: bool,
    dot_matches_new_line: bool,
}

impl Regex {
    pub(crate) fn new(pattern: &str) -> Result<Self, RegexError> {
        RegexBuilder::new(pattern).build()
    }

    pub(crate) fn is_match(&self, input: &str) -> Result<bool, RegexError> {
        Ok(self.find(input)?.is_some())
    }

    pub(crate) fn captures(&self, input: &str) -> Result<Option<Captures>, RegexError> {
        self.captures_from_pos(input, 0)
    }

    pub(crate) fn captures_from_pos(
        &self,
        input: &str,
        start: usize,
    ) -> Result<Option<Captures>, RegexError> {
        let prepared = PreparedInput::new(input);
        if start > prepared.text_len_bytes() {
            return Ok(None);
        }
        let Some(start_char) = prepared.byte_to_char_index(start) else {
            return Err(RegexError::new(
                "search start is not a valid UTF-8 character boundary",
            ));
        };
        Ok(self
            .find_spans_from_char(&prepared, start_char)
            .map(|spans| self.spans_to_captures(&prepared, spans)))
    }

    pub(crate) fn find(&self, input: &str) -> Result<Option<Match>, RegexError> {
        let captures = self.captures(input)?;
        Ok(captures.and_then(|c| c.get(0).cloned()))
    }

    fn find_spans_from_char(
        &self,
        prepared: &PreparedInput<'_>,
        start_char: usize,
    ) -> Option<Vec<Option<(usize, usize)>>> {
        let opts = MatchOptions {
            ignore_case: self.ignore_case,
            multi_line: self.multi_line,
            dot_matches_new_line: self.dot_matches_new_line,
        };

        let max_start = prepared.text_len_chars();
        for candidate_start in start_char..=max_start {
            let mut caps = vec![None; self.capture_count + 1];
            let initial = MatchState {
                pos: candidate_start,
                captures: caps.split_off(0),
            };
            let states = self.ast.match_states(prepared, &initial, &opts);
            if states.is_empty() {
                continue;
            }
            let mut first = states[0].clone();
            first.captures[0] = Some((candidate_start, first.pos));
            return Some(first.captures);
        }
        None
    }

    fn spans_to_captures(
        &self,
        prepared: &PreparedInput<'_>,
        spans: Vec<Option<(usize, usize)>>,
    ) -> Captures {
        let mut groups = Vec::with_capacity(spans.len());
        for span in spans {
            if let Some((start_char, end_char)) = span {
                let start = prepared.char_to_byte_index(start_char);
                let end = prepared.char_to_byte_index(end_char);
                let text = prepared.slice_bytes_to_string(start, end);
                groups.push(Some(Match { start, end, text }));
            } else {
                groups.push(None);
            }
        }
        Captures {
            groups,
            named_captures: self.named_captures.clone(),
            named_capture_order: self.named_capture_order.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RegexBuilder {
    pattern: String,
    case_insensitive: bool,
    multi_line: bool,
    dot_matches_new_line: bool,
    unicode_mode: bool,
}

impl RegexBuilder {
    pub(crate) fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            case_insensitive: false,
            multi_line: false,
            dot_matches_new_line: false,
            unicode_mode: false,
        }
    }

    pub(crate) fn case_insensitive(&mut self, enabled: bool) -> &mut Self {
        self.case_insensitive = enabled;
        self
    }

    pub(crate) fn multi_line(&mut self, enabled: bool) -> &mut Self {
        self.multi_line = enabled;
        self
    }

    pub(crate) fn dot_matches_new_line(&mut self, enabled: bool) -> &mut Self {
        self.dot_matches_new_line = enabled;
        self
    }

    pub(crate) fn unicode_mode(&mut self, enabled: bool) -> &mut Self {
        self.unicode_mode = enabled;
        self
    }

    pub(crate) fn build(&self) -> Result<Regex, RegexError> {
        let mut parser = Parser::new(&self.pattern, self.unicode_mode);
        let ast = parser.parse()?;
        Ok(Regex {
            ast,
            capture_count: parser.capture_count,
            named_captures: parser.named_captures,
            named_capture_order: parser.named_capture_order,
            ignore_case: self.case_insensitive,
            multi_line: self.multi_line,
            dot_matches_new_line: self.dot_matches_new_line,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Captures {
    groups: Vec<Option<Match>>,
    named_captures: HashMap<String, usize>,
    named_capture_order: Vec<String>,
}

impl Captures {
    pub(crate) fn len(&self) -> usize {
        self.groups.len()
    }

    pub(crate) fn get(&self, index: usize) -> Option<&Match> {
        self.groups.get(index).and_then(Option::as_ref)
    }

    pub(crate) fn get_named(&self, name: &str) -> Option<&Match> {
        let index = self.named_captures.get(name).copied()?;
        self.get(index)
    }

    pub(crate) fn has_named_groups(&self) -> bool {
        !self.named_captures.is_empty()
    }

    pub(crate) fn named_group_names(&self) -> Vec<String> {
        self.named_capture_order.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Match {
    start: usize,
    end: usize,
    text: String,
}

impl Match {
    pub(crate) fn as_str(&self) -> &str {
        self.text.as_str()
    }

    pub(crate) fn start(&self) -> usize {
        self.start
    }

    pub(crate) fn end(&self) -> usize {
        self.end
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RegexError {
    message: String,
}

impl RegexError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RegexError {}

pub(crate) fn escape(value: &str) -> Cow<'_, str> {
    let mut out = String::with_capacity(value.len());
    let mut changed = false;

    for ch in value.chars() {
        if is_regex_meta(ch) {
            out.push('\\');
            changed = true;
        }
        out.push(ch);
    }

    if changed {
        Cow::Owned(out)
    } else {
        Cow::Borrowed(value)
    }
}

#[derive(Debug, Clone)]
struct PreparedInput<'a> {
    src: &'a str,
    chars: Vec<char>,
    char_to_byte: Vec<usize>,
}

impl<'a> PreparedInput<'a> {
    fn new(src: &'a str) -> Self {
        let mut chars = Vec::new();
        let mut char_to_byte = Vec::new();
        for (idx, ch) in src.char_indices() {
            chars.push(ch);
            char_to_byte.push(idx);
        }
        char_to_byte.push(src.len());
        Self {
            src,
            chars,
            char_to_byte,
        }
    }

    fn text_len_chars(&self) -> usize {
        self.chars.len()
    }

    fn text_len_bytes(&self) -> usize {
        self.src.len()
    }

    fn char_at(&self, idx: usize) -> Option<char> {
        self.chars.get(idx).copied()
    }

    fn char_to_byte_index(&self, idx: usize) -> usize {
        self.char_to_byte
            .get(idx)
            .copied()
            .unwrap_or(self.src.len())
    }

    fn byte_to_char_index(&self, byte_idx: usize) -> Option<usize> {
        self.char_to_byte.binary_search(&byte_idx).ok()
    }

    fn slice_bytes_to_string(&self, start: usize, end: usize) -> String {
        self.src.get(start..end).unwrap_or_default().to_string()
    }

    fn is_word_boundary(&self, pos: usize) -> bool {
        let prev = pos
            .checked_sub(1)
            .and_then(|idx| self.char_at(idx))
            .is_some_and(is_word_char);
        let next = self.char_at(pos).is_some_and(is_word_char);
        prev != next
    }
}

#[derive(Debug, Clone)]
struct MatchState {
    pos: usize,
    captures: Vec<Option<(usize, usize)>>,
}

#[derive(Debug, Clone, Copy)]
struct MatchOptions {
    ignore_case: bool,
    multi_line: bool,
    dot_matches_new_line: bool,
}

#[derive(Debug, Clone)]
enum Expr {
    Empty,
    Sequence(Vec<Expr>),
    Alternation(Vec<Expr>),
    Literal(char),
    Dot,
    StartAnchor,
    EndAnchor,
    WordBoundary(bool),
    CharClass(CharClass),
    Group {
        index: Option<usize>,
        expr: Box<Expr>,
    },
    LookAhead {
        positive: bool,
        expr: Box<Expr>,
    },
    LookBehind {
        positive: bool,
        expr: Box<Expr>,
    },
    NamedBackReference(String),
    BackReference(usize),
    Quantifier {
        expr: Box<Expr>,
        min: usize,
        max: Option<usize>,
        greedy: bool,
    },
}

impl Expr {
    fn match_states(
        &self,
        prepared: &PreparedInput<'_>,
        state: &MatchState,
        opts: &MatchOptions,
    ) -> Vec<MatchState> {
        match self {
            Self::Empty => vec![state.clone()],
            Self::Sequence(parts) => {
                let mut states = vec![state.clone()];
                for part in parts {
                    let mut next = Vec::new();
                    for candidate in &states {
                        next.extend(part.match_states(prepared, candidate, opts));
                    }
                    if next.is_empty() {
                        return Vec::new();
                    }
                    states = next;
                }
                states
            }
            Self::Alternation(branches) => {
                let mut out = Vec::new();
                for branch in branches {
                    out.extend(branch.match_states(prepared, state, opts));
                }
                out
            }
            Self::Literal(expected) => {
                let Some(actual) = prepared.char_at(state.pos) else {
                    return Vec::new();
                };
                if chars_equal(actual, *expected, opts.ignore_case) {
                    let mut next = state.clone();
                    next.pos += 1;
                    vec![next]
                } else {
                    Vec::new()
                }
            }
            Self::Dot => {
                let Some(actual) = prepared.char_at(state.pos) else {
                    return Vec::new();
                };
                if opts.dot_matches_new_line || !is_line_terminator(actual) {
                    let mut next = state.clone();
                    next.pos += 1;
                    vec![next]
                } else {
                    Vec::new()
                }
            }
            Self::StartAnchor => {
                let at_start = state.pos == 0;
                let at_line_start = opts.multi_line
                    && state
                        .pos
                        .checked_sub(1)
                        .and_then(|idx| prepared.char_at(idx))
                        .is_some_and(is_line_terminator);
                if at_start || at_line_start {
                    vec![state.clone()]
                } else {
                    Vec::new()
                }
            }
            Self::EndAnchor => {
                let at_end = state.pos == prepared.text_len_chars();
                let at_line_end =
                    opts.multi_line && prepared.char_at(state.pos).is_some_and(is_line_terminator);
                if at_end || at_line_end {
                    vec![state.clone()]
                } else {
                    Vec::new()
                }
            }
            Self::WordBoundary(expected) => {
                if prepared.is_word_boundary(state.pos) == *expected {
                    vec![state.clone()]
                } else {
                    Vec::new()
                }
            }
            Self::CharClass(class) => {
                let Some(actual) = prepared.char_at(state.pos) else {
                    return Vec::new();
                };
                if class.matches(actual, opts.ignore_case) {
                    let mut next = state.clone();
                    next.pos += 1;
                    vec![next]
                } else {
                    Vec::new()
                }
            }
            Self::Group { index, expr } => {
                let start = state.pos;
                let states = expr.match_states(prepared, state, opts);
                let mut out = Vec::with_capacity(states.len());
                for mut candidate in states {
                    if let Some(idx) = index {
                        if let Some(slot) = candidate.captures.get_mut(*idx) {
                            *slot = Some((start, candidate.pos));
                        }
                    }
                    out.push(candidate);
                }
                out
            }
            Self::LookAhead { positive, expr } => {
                let inner = expr.match_states(prepared, state, opts);
                if (*positive && !inner.is_empty()) || (!*positive && inner.is_empty()) {
                    vec![state.clone()]
                } else {
                    Vec::new()
                }
            }
            Self::LookBehind { positive, expr } => {
                let mut matched = false;
                for candidate_start in 0..=state.pos {
                    let mut candidate = state.clone();
                    candidate.pos = candidate_start;
                    let inner = expr.match_states(prepared, &candidate, opts);
                    if inner.iter().any(|next| next.pos == state.pos) {
                        matched = true;
                        break;
                    }
                }
                if (*positive && matched) || (!*positive && !matched) {
                    vec![state.clone()]
                } else {
                    Vec::new()
                }
            }
            Self::BackReference(index) => {
                let Some(group) = state.captures.get(*index).copied().flatten() else {
                    return vec![state.clone()];
                };
                let len = group.1.saturating_sub(group.0);
                if state.pos.saturating_add(len) > prepared.text_len_chars() {
                    return Vec::new();
                }
                for offset in 0..len {
                    let Some(left) = prepared.char_at(group.0 + offset) else {
                        return Vec::new();
                    };
                    let Some(right) = prepared.char_at(state.pos + offset) else {
                        return Vec::new();
                    };
                    if !chars_equal(left, right, opts.ignore_case) {
                        return Vec::new();
                    }
                }
                let mut next = state.clone();
                next.pos += len;
                vec![next]
            }
            Self::NamedBackReference(_) => Vec::new(),
            Self::Quantifier {
                expr,
                min,
                max,
                greedy,
            } => {
                let mut levels = vec![vec![state.clone()]];
                let limit = max.unwrap_or(usize::MAX);

                while levels.len() - 1 < limit {
                    let prev = levels.last().expect("levels always has one element");
                    let mut next = Vec::new();
                    let mut saw_progress = false;
                    for candidate in prev {
                        for matched in expr.match_states(prepared, candidate, opts) {
                            if matched.pos != candidate.pos {
                                saw_progress = true;
                            }
                            next.push(matched);
                        }
                    }
                    if next.is_empty() {
                        break;
                    }

                    if !saw_progress {
                        let repeated = next.clone();
                        levels.push(next);
                        let target = match max {
                            Some(max) if *max == *min => *min,
                            Some(max) => min.saturating_add(1).min(*max),
                            None => min.saturating_add(1),
                        };
                        while levels.len() - 1 < target {
                            levels.push(repeated.clone());
                        }
                        break;
                    }

                    levels.push(next);
                }

                let available_max = levels.len().saturating_sub(1);
                if *min > available_max {
                    return Vec::new();
                }

                let mut out = Vec::new();
                if *greedy {
                    for count in (*min..=available_max).rev() {
                        out.extend(levels[count].iter().cloned());
                    }
                } else {
                    for count in *min..=available_max {
                        out.extend(levels[count].iter().cloned());
                    }
                }
                out
            }
        }
    }
}

#[derive(Debug, Clone)]
struct CharClass {
    negated: bool,
    items: Vec<ClassItem>,
}

impl CharClass {
    fn matches(&self, ch: char, ignore_case: bool) -> bool {
        let matched = self.items.iter().any(|item| item.matches(ch, ignore_case));
        if self.negated { !matched } else { matched }
    }
}

#[derive(Debug, Clone)]
enum ClassItem {
    Char(char),
    Range(char, char),
    Digit,
    NotDigit,
    Word,
    NotWord,
    Space,
    NotSpace,
}

impl ClassItem {
    fn matches(&self, ch: char, ignore_case: bool) -> bool {
        match self {
            Self::Char(expected) => chars_equal(ch, *expected, ignore_case),
            Self::Range(start, end) => char_in_range(ch, *start, *end, ignore_case),
            Self::Digit => ch.is_ascii_digit(),
            Self::NotDigit => !ch.is_ascii_digit(),
            Self::Word => is_word_char(ch),
            Self::NotWord => !is_word_char(ch),
            Self::Space => is_space_char(ch),
            Self::NotSpace => !is_space_char(ch),
        }
    }
}

struct Parser {
    chars: Vec<char>,
    i: usize,
    unicode_mode: bool,
    capture_count: usize,
    named_captures: HashMap<String, usize>,
    named_capture_order: Vec<String>,
}

impl Parser {
    fn new(pattern: &str, unicode_mode: bool) -> Self {
        Self {
            chars: pattern.chars().collect(),
            i: 0,
            unicode_mode,
            capture_count: 0,
            named_captures: HashMap::new(),
            named_capture_order: Vec::new(),
        }
    }

    fn parse(&mut self) -> Result<Expr, RegexError> {
        let mut expr = self.parse_alternation()?;
        if self.i < self.chars.len() {
            return Err(RegexError::new("invalid regular expression syntax"));
        }
        self.resolve_named_backreferences(&mut expr)?;
        Ok(expr)
    }

    fn parse_alternation(&mut self) -> Result<Expr, RegexError> {
        let mut branches = vec![self.parse_sequence()?];
        while self.peek() == Some('|') {
            self.i += 1;
            branches.push(self.parse_sequence()?);
        }
        if branches.len() == 1 {
            Ok(branches.pop().unwrap_or(Expr::Empty))
        } else {
            Ok(Expr::Alternation(branches))
        }
    }

    fn parse_sequence(&mut self) -> Result<Expr, RegexError> {
        let mut items = Vec::new();
        while let Some(ch) = self.peek() {
            if ch == ')' || ch == '|' {
                break;
            }
            let atom = self.parse_atom()?;
            let quantified = self.parse_quantifier(atom)?;
            items.push(quantified);
        }

        if items.is_empty() {
            Ok(Expr::Empty)
        } else if items.len() == 1 {
            Ok(items.pop().unwrap_or(Expr::Empty))
        } else {
            Ok(Expr::Sequence(items))
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, RegexError> {
        let Some(ch) = self.next() else {
            return Err(RegexError::new("unexpected end of regular expression"));
        };

        match ch {
            '^' => Ok(Expr::StartAnchor),
            '$' => Ok(Expr::EndAnchor),
            '.' => Ok(Expr::Dot),
            '(' => self.parse_group(),
            '[' => self.parse_char_class(),
            '\\' => self.parse_escape(false),
            _ => Ok(Expr::Literal(ch)),
        }
    }

    fn parse_group(&mut self) -> Result<Expr, RegexError> {
        if self.peek() == Some('?') {
            self.i += 1;
            match self.next() {
                Some(':') => {
                    let expr = self.parse_alternation()?;
                    self.expect(')')?;
                    Ok(Expr::Group {
                        index: None,
                        expr: Box::new(expr),
                    })
                }
                Some('=') => {
                    let expr = self.parse_alternation()?;
                    self.expect(')')?;
                    Ok(Expr::LookAhead {
                        positive: true,
                        expr: Box::new(expr),
                    })
                }
                Some('!') => {
                    let expr = self.parse_alternation()?;
                    self.expect(')')?;
                    Ok(Expr::LookAhead {
                        positive: false,
                        expr: Box::new(expr),
                    })
                }
                Some('<') => {
                    if self.peek() == Some('=') {
                        self.i += 1;
                        let expr = self.parse_alternation()?;
                        self.expect(')')?;
                        Ok(Expr::LookBehind {
                            positive: true,
                            expr: Box::new(expr),
                        })
                    } else if self.peek() == Some('!') {
                        self.i += 1;
                        let expr = self.parse_alternation()?;
                        self.expect(')')?;
                        Ok(Expr::LookBehind {
                            positive: false,
                            expr: Box::new(expr),
                        })
                    } else {
                        let name = self.parse_group_name()?;
                        self.capture_count += 1;
                        let index = self.capture_count;
                        if self.named_captures.insert(name.clone(), index).is_some() {
                            return Err(RegexError::new(format!(
                                "duplicate capture group name: {name}"
                            )));
                        }
                        self.named_capture_order.push(name);
                        let expr = self.parse_alternation()?;
                        self.expect(')')?;
                        Ok(Expr::Group {
                            index: Some(index),
                            expr: Box::new(expr),
                        })
                    }
                }
                _ => Err(RegexError::new("unsupported group syntax")),
            }
        } else {
            self.capture_count += 1;
            let index = self.capture_count;
            let expr = self.parse_alternation()?;
            self.expect(')')?;
            Ok(Expr::Group {
                index: Some(index),
                expr: Box::new(expr),
            })
        }
    }

    fn parse_quantifier(&mut self, atom: Expr) -> Result<Expr, RegexError> {
        let Some(ch) = self.peek() else {
            return Ok(atom);
        };

        let quant = match ch {
            '*' => {
                self.i += 1;
                Some((0usize, None))
            }
            '+' => {
                self.i += 1;
                Some((1usize, None))
            }
            '?' => {
                self.i += 1;
                Some((0usize, Some(1usize)))
            }
            '{' => Some(self.parse_braced_quantifier()?),
            _ => None,
        };

        let Some((min, max)) = quant else {
            return Ok(atom);
        };

        if !is_quantifier_target_supported(&atom) {
            return Err(RegexError::new("nothing to repeat"));
        }
        if self.unicode_mode && matches!(atom, Expr::LookAhead { .. }) {
            return Err(RegexError::new("invalid quantifier"));
        }

        let greedy = if self.peek() == Some('?') {
            self.i += 1;
            false
        } else {
            true
        };

        Ok(Expr::Quantifier {
            expr: Box::new(atom),
            min,
            max,
            greedy,
        })
    }

    fn parse_braced_quantifier(&mut self) -> Result<(usize, Option<usize>), RegexError> {
        self.expect('{')?;
        let min = self.parse_usize()?;

        match self.peek() {
            Some('}') => {
                self.i += 1;
                Ok((min, Some(min)))
            }
            Some(',') => {
                self.i += 1;
                if self.peek() == Some('}') {
                    self.i += 1;
                    Ok((min, None))
                } else {
                    let max = self.parse_usize()?;
                    if max < min {
                        return Err(RegexError::new("invalid quantifier range"));
                    }
                    self.expect('}')?;
                    Ok((min, Some(max)))
                }
            }
            _ => Err(RegexError::new("invalid quantifier syntax")),
        }
    }

    fn parse_char_class(&mut self) -> Result<Expr, RegexError> {
        let mut negated = false;
        if self.peek() == Some('^') {
            negated = true;
            self.i += 1;
        }

        let mut items = Vec::new();
        let mut saw_any = false;

        while let Some(ch) = self.peek() {
            if ch == ']' {
                self.i += 1;
                if !saw_any {
                    return Err(RegexError::new("empty character class"));
                }
                return Ok(Expr::CharClass(CharClass { negated, items }));
            }
            saw_any = true;

            let item = self.parse_class_item()?;
            if self.peek() == Some('-') {
                let backup = self.i;
                self.i += 1;
                if self.peek() == Some(']') {
                    self.i = backup;
                    items.push(item);
                    continue;
                }
                let next_item = self.parse_class_item()?;
                match (item, next_item) {
                    (ClassItem::Char(start), ClassItem::Char(end)) => {
                        items.push(ClassItem::Range(start, end));
                    }
                    _ => {
                        return Err(RegexError::new(
                            "character class range bounds must be literal characters",
                        ));
                    }
                }
            } else {
                items.push(item);
            }
        }

        Err(RegexError::new("unterminated character class"))
    }

    fn parse_class_item(&mut self) -> Result<ClassItem, RegexError> {
        let Some(ch) = self.next() else {
            return Err(RegexError::new("unterminated character class"));
        };

        if ch != '\\' {
            return Ok(ClassItem::Char(ch));
        }

        let Some(escaped) = self.next() else {
            return Err(RegexError::new("unterminated escape sequence"));
        };

        match escaped {
            'd' => Ok(ClassItem::Digit),
            'D' => Ok(ClassItem::NotDigit),
            'w' => Ok(ClassItem::Word),
            'W' => Ok(ClassItem::NotWord),
            's' => Ok(ClassItem::Space),
            'S' => Ok(ClassItem::NotSpace),
            'c' => {
                if self.peek().is_some_and(|next| next.is_ascii_alphabetic()) {
                    let next = self.next().expect("peek guaranteed a character");
                    Ok(ClassItem::Char(control_escape_char(next)))
                } else {
                    Ok(ClassItem::Char('c'))
                }
            }
            'n' => Ok(ClassItem::Char('\n')),
            'r' => Ok(ClassItem::Char('\r')),
            't' => Ok(ClassItem::Char('\t')),
            'v' => Ok(ClassItem::Char('\u{000B}')),
            'f' => Ok(ClassItem::Char('\u{000C}')),
            'b' => Ok(ClassItem::Char('\u{0008}')),
            '0' => Ok(ClassItem::Char('\0')),
            'x' => Ok(ClassItem::Char(self.parse_hex_char(2)?)),
            'u' => Ok(ClassItem::Char(self.parse_unicode_escape()?)),
            other => Ok(ClassItem::Char(other)),
        }
    }

    fn parse_escape(&mut self, _in_class: bool) -> Result<Expr, RegexError> {
        let Some(ch) = self.next() else {
            return Err(RegexError::new("unterminated escape sequence"));
        };

        match ch {
            'd' => Ok(Expr::CharClass(CharClass {
                negated: false,
                items: vec![ClassItem::Digit],
            })),
            'D' => Ok(Expr::CharClass(CharClass {
                negated: false,
                items: vec![ClassItem::NotDigit],
            })),
            'w' => Ok(Expr::CharClass(CharClass {
                negated: false,
                items: vec![ClassItem::Word],
            })),
            'W' => Ok(Expr::CharClass(CharClass {
                negated: false,
                items: vec![ClassItem::NotWord],
            })),
            's' => Ok(Expr::CharClass(CharClass {
                negated: false,
                items: vec![ClassItem::Space],
            })),
            'S' => Ok(Expr::CharClass(CharClass {
                negated: false,
                items: vec![ClassItem::NotSpace],
            })),
            'b' => Ok(Expr::WordBoundary(true)),
            'B' => Ok(Expr::WordBoundary(false)),
            'n' => Ok(Expr::Literal('\n')),
            'r' => Ok(Expr::Literal('\r')),
            't' => Ok(Expr::Literal('\t')),
            'v' => Ok(Expr::Literal('\u{000B}')),
            'f' => Ok(Expr::Literal('\u{000C}')),
            'c' => {
                if self.peek().is_some_and(|next| next.is_ascii_alphabetic()) {
                    let next = self.next().expect("peek guaranteed a character");
                    Ok(Expr::Literal(control_escape_char(next)))
                } else {
                    Ok(Expr::Literal('c'))
                }
            }
            '0' => Ok(Expr::Literal('\0')),
            'x' => Ok(Expr::Literal(self.parse_hex_char(2)?)),
            'u' => Ok(Expr::Literal(self.parse_unicode_escape()?)),
            'k' => {
                if self.next() != Some('<') {
                    return Err(RegexError::new("invalid named backreference syntax"));
                }
                let name = self.parse_group_name()?;
                Ok(Expr::NamedBackReference(name))
            }
            '1'..='9' => self.parse_numeric_backreference(ch),
            other => Ok(Expr::Literal(other)),
        }
    }

    fn parse_group_name(&mut self) -> Result<String, RegexError> {
        let start = self.i;
        while let Some(ch) = self.peek() {
            if ch == '>' {
                break;
            }
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.i += 1;
            } else {
                return Err(RegexError::new("invalid capture group name"));
            }
        }
        if self.peek() != Some('>') || self.i == start {
            return Err(RegexError::new("invalid capture group name"));
        }
        let name: String = self.chars[start..self.i].iter().collect();
        self.i += 1;
        Ok(name)
    }

    fn parse_numeric_backreference(&mut self, first_digit: char) -> Result<Expr, RegexError> {
        let mut raw = String::new();
        raw.push(first_digit);
        while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            raw.push(self.next().expect("peek guaranteed a digit"));
        }
        let index = raw
            .parse::<usize>()
            .map_err(|_| RegexError::new("invalid numeric backreference"))?;
        if index == 0 || index > self.capture_count {
            return Err(RegexError::new("invalid numeric backreference"));
        }
        Ok(Expr::BackReference(index))
    }

    fn parse_unicode_escape(&mut self) -> Result<char, RegexError> {
        if self.peek() == Some('{') {
            self.i += 1;
            let start = self.i;
            while self.peek().is_some_and(|ch| ch.is_ascii_hexdigit()) {
                self.i += 1;
            }
            if self.peek() != Some('}') || self.i == start {
                return Err(RegexError::new("invalid Unicode escape sequence"));
            }
            let raw: String = self.chars[start..self.i].iter().collect();
            self.i += 1;
            let value = u32::from_str_radix(&raw, 16)
                .map_err(|_| RegexError::new("invalid Unicode escape sequence"))?;
            char::from_u32(value).ok_or_else(|| RegexError::new("invalid Unicode escape sequence"))
        } else {
            self.parse_hex_char(4)
        }
    }

    fn parse_hex_char(&mut self, digits: usize) -> Result<char, RegexError> {
        let start = self.i;
        let end = start.saturating_add(digits);
        if end > self.chars.len() {
            return Err(RegexError::new("invalid hexadecimal escape sequence"));
        }
        if !self.chars[start..end]
            .iter()
            .all(|ch| ch.is_ascii_hexdigit())
        {
            return Err(RegexError::new("invalid hexadecimal escape sequence"));
        }
        let raw: String = self.chars[start..end].iter().collect();
        self.i = end;
        let value = u32::from_str_radix(&raw, 16)
            .map_err(|_| RegexError::new("invalid hexadecimal escape sequence"))?;
        char::from_u32(value).ok_or_else(|| RegexError::new("invalid escape value"))
    }

    fn parse_usize(&mut self) -> Result<usize, RegexError> {
        let start = self.i;
        while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            self.i += 1;
        }
        if self.i == start {
            return Err(RegexError::new("expected number in quantifier"));
        }
        let raw: String = self.chars[start..self.i].iter().collect();
        raw.parse::<usize>()
            .map_err(|_| RegexError::new("invalid numeric value in quantifier"))
    }

    fn expect(&mut self, ch: char) -> Result<(), RegexError> {
        if self.next() == Some(ch) {
            Ok(())
        } else {
            Err(RegexError::new("unterminated group or invalid syntax"))
        }
    }

    fn resolve_named_backreferences(&self, expr: &mut Expr) -> Result<(), RegexError> {
        match expr {
            Expr::NamedBackReference(name) => {
                let Some(index) = self.named_captures.get(name).copied() else {
                    return Err(RegexError::new(format!(
                        "unknown named backreference: {name}"
                    )));
                };
                *expr = Expr::BackReference(index);
                Ok(())
            }
            Expr::Sequence(parts) | Expr::Alternation(parts) => {
                for part in parts {
                    self.resolve_named_backreferences(part)?;
                }
                Ok(())
            }
            Expr::Group { expr, .. }
            | Expr::LookAhead { expr, .. }
            | Expr::LookBehind { expr, .. }
            | Expr::Quantifier { expr, .. } => self.resolve_named_backreferences(expr),
            _ => Ok(()),
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).copied()
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.i += 1;
        Some(ch)
    }
}

fn chars_equal(left: char, right: char, ignore_case: bool) -> bool {
    if !ignore_case {
        return left == right;
    }
    if left.is_ascii() && right.is_ascii() {
        left.eq_ignore_ascii_case(&right)
    } else {
        left.to_lowercase().eq(right.to_lowercase())
    }
}

fn control_escape_char(letter: char) -> char {
    let upper = letter.to_ascii_uppercase() as u8;
    char::from(upper - b'@')
}

fn is_quantifier_target_supported(expr: &Expr) -> bool {
    !matches!(
        expr,
        Expr::StartAnchor | Expr::EndAnchor | Expr::WordBoundary(_) | Expr::LookBehind { .. }
    )
}

fn char_in_range(ch: char, start: char, end: char, ignore_case: bool) -> bool {
    let (start_code, end_code, code) =
        if ignore_case && start.is_ascii() && end.is_ascii() && ch.is_ascii() {
            (
                (start as u8).to_ascii_lowercase() as u32,
                (end as u8).to_ascii_lowercase() as u32,
                (ch as u8).to_ascii_lowercase() as u32,
            )
        } else {
            (start as u32, end as u32, ch as u32)
        };
    if start_code <= end_code {
        (start_code..=end_code).contains(&code)
    } else {
        (end_code..=start_code).contains(&code)
    }
}

fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn is_space_char(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '\r' | '\u{000B}' | '\u{000C}') || ch.is_whitespace()
}

fn is_line_terminator(ch: char) -> bool {
    matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

fn is_regex_meta(ch: char) -> bool {
    matches!(
        ch,
        '\\' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '^' | '$' | '/'
    )
}
