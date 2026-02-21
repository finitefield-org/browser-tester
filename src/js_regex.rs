use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
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
    unicode_sets_mode: bool,
}

impl RegexBuilder {
    pub(crate) fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            case_insensitive: false,
            multi_line: false,
            dot_matches_new_line: false,
            unicode_mode: false,
            unicode_sets_mode: false,
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

    pub(crate) fn unicode_sets_mode(&mut self, enabled: bool) -> &mut Self {
        self.unicode_sets_mode = enabled;
        self
    }

    pub(crate) fn build(&self) -> Result<Regex, RegexError> {
        let mut parser = Parser::new(&self.pattern, self.unicode_mode, self.unicode_sets_mode);
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
    UnicodeStringProperty(UnicodeStringProperty),
    ClassSetOperation {
        operands: Vec<CharClass>,
        operators: Vec<ClassSetOperator>,
        negated: bool,
    },
    Group {
        index: Option<usize>,
        expr: Box<Expr>,
    },
    FlagModifierGroup {
        expr: Box<Expr>,
        ignore_case: Option<bool>,
        multi_line: Option<bool>,
        dot_matches_new_line: Option<bool>,
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
    DecimalEscape(String),
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
            Self::UnicodeStringProperty(property) => {
                let Some(next_pos) = property.match_at(prepared, state.pos) else {
                    return Vec::new();
                };
                let mut next = state.clone();
                next.pos = next_pos;
                vec![next]
            }
            Self::ClassSetOperation {
                operands,
                operators,
                negated,
            } => {
                let ends = class_set_operation_match_ends(
                    prepared,
                    state.pos,
                    operands,
                    operators,
                    *negated,
                    opts.ignore_case,
                );
                if ends.is_empty() {
                    return Vec::new();
                }
                let mut out = Vec::with_capacity(ends.len());
                for end in ends {
                    let mut next = state.clone();
                    next.pos = end;
                    out.push(next);
                }
                out
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
            Self::FlagModifierGroup {
                expr,
                ignore_case,
                multi_line,
                dot_matches_new_line,
            } => {
                let mut local_opts = *opts;
                if let Some(value) = ignore_case {
                    local_opts.ignore_case = *value;
                }
                if let Some(value) = multi_line {
                    local_opts.multi_line = *value;
                }
                if let Some(value) = dot_matches_new_line {
                    local_opts.dot_matches_new_line = *value;
                }
                expr.match_states(prepared, state, &local_opts)
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
            Self::DecimalEscape(_) => Vec::new(),
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

    fn match_ends(
        &self,
        prepared: &PreparedInput<'_>,
        pos: usize,
        ignore_case: bool,
    ) -> Vec<usize> {
        if self.negated {
            let Some(actual) = prepared.char_at(pos) else {
                return Vec::new();
            };
            if self.matches(actual, ignore_case) {
                Vec::new()
            } else {
                vec![pos + 1]
            }
        } else {
            let mut out = Vec::new();
            for item in &self.items {
                out.extend(item.match_ends(prepared, pos, ignore_case));
            }
            dedupe_sorted_desc(out)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClassSetOperator {
    Intersection,
    Difference,
}

#[derive(Debug, Clone)]
enum ClassItem {
    Char(char),
    Range(char, char),
    NestedExpression(Box<Expr>),
    StringDisjunction(Vec<String>),
    Digit,
    NotDigit,
    Word,
    NotWord,
    Space,
    NotSpace,
    UnicodeProperty {
        property: UnicodeProperty,
        negated: bool,
    },
    UnicodeStringProperty(UnicodeStringProperty),
}

impl ClassItem {
    fn matches(&self, ch: char, ignore_case: bool) -> bool {
        match self {
            Self::Char(expected) => chars_equal(ch, *expected, ignore_case),
            Self::Range(start, end) => char_in_range(ch, *start, *end, ignore_case),
            Self::NestedExpression(expr) => expr_matches_single_char(expr, ch, ignore_case),
            Self::StringDisjunction(alternatives) => alternatives.iter().any(|value| {
                single_char_string(value)
                    .is_some_and(|expected| chars_equal(ch, expected, ignore_case))
            }),
            Self::Digit => ch.is_ascii_digit(),
            Self::NotDigit => !ch.is_ascii_digit(),
            Self::Word => is_word_char(ch),
            Self::NotWord => !is_word_char(ch),
            Self::Space => is_space_char(ch),
            Self::NotSpace => !is_space_char(ch),
            Self::UnicodeProperty { property, negated } => {
                let matched = property.matches(ch);
                if *negated { !matched } else { matched }
            }
            Self::UnicodeStringProperty(_) => false,
        }
    }

    fn match_ends(
        &self,
        prepared: &PreparedInput<'_>,
        pos: usize,
        ignore_case: bool,
    ) -> Vec<usize> {
        match self {
            Self::Char(expected) => prepared
                .char_at(pos)
                .filter(|actual| chars_equal(*actual, *expected, ignore_case))
                .map(|_| pos + 1)
                .into_iter()
                .collect::<Vec<_>>(),
            Self::Range(start, end) => prepared
                .char_at(pos)
                .filter(|actual| char_in_range(*actual, *start, *end, ignore_case))
                .map(|_| pos + 1)
                .into_iter()
                .collect::<Vec<_>>(),
            Self::NestedExpression(expr) => expr_match_ends(expr, prepared, pos, ignore_case),
            Self::StringDisjunction(alternatives) => alternatives
                .iter()
                .filter_map(|candidate| match_literal_string_at(prepared, pos, candidate, ignore_case))
                .collect::<Vec<_>>(),
            Self::Digit => prepared
                .char_at(pos)
                .filter(|actual| actual.is_ascii_digit())
                .map(|_| pos + 1)
                .into_iter()
                .collect::<Vec<_>>(),
            Self::NotDigit => prepared
                .char_at(pos)
                .filter(|actual| !actual.is_ascii_digit())
                .map(|_| pos + 1)
                .into_iter()
                .collect::<Vec<_>>(),
            Self::Word => prepared
                .char_at(pos)
                .filter(|actual| is_word_char(*actual))
                .map(|_| pos + 1)
                .into_iter()
                .collect::<Vec<_>>(),
            Self::NotWord => prepared
                .char_at(pos)
                .filter(|actual| !is_word_char(*actual))
                .map(|_| pos + 1)
                .into_iter()
                .collect::<Vec<_>>(),
            Self::Space => prepared
                .char_at(pos)
                .filter(|actual| is_space_char(*actual))
                .map(|_| pos + 1)
                .into_iter()
                .collect::<Vec<_>>(),
            Self::NotSpace => prepared
                .char_at(pos)
                .filter(|actual| !is_space_char(*actual))
                .map(|_| pos + 1)
                .into_iter()
                .collect::<Vec<_>>(),
            Self::UnicodeProperty { property, negated } => prepared
                .char_at(pos)
                .filter(|actual| {
                    let matched = property.matches(*actual);
                    if *negated { !matched } else { matched }
                })
                .map(|_| pos + 1)
                .into_iter()
                .collect::<Vec<_>>(),
            Self::UnicodeStringProperty(property) => property
                .match_at(prepared, pos)
                .into_iter()
                .collect::<Vec<_>>(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum UnicodeProperty {
    Letter,
    Number,
    DecimalNumber,
    Ascii,
    Any,
    ScriptLatin,
    ScriptGreek,
}

impl UnicodeProperty {
    fn matches(&self, ch: char) -> bool {
        match self {
            Self::Letter => ch.is_alphabetic(),
            Self::Number => ch.is_numeric(),
            Self::DecimalNumber => char_in_ranges(ch, UNICODE_DECIMAL_NUMBER_RANGES),
            Self::Ascii => ch.is_ascii(),
            Self::Any => true,
            Self::ScriptLatin => char_in_ranges(ch, UNICODE_SCRIPT_LATIN_RANGES),
            Self::ScriptGreek => char_in_ranges(ch, UNICODE_SCRIPT_GREEK_RANGES),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum UnicodeStringProperty {
    RgiEmoji,
}

impl UnicodeStringProperty {
    fn match_at(&self, prepared: &PreparedInput<'_>, pos: usize) -> Option<usize> {
        match self {
            Self::RgiEmoji => match_rgi_emoji_at(prepared, pos),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum UnicodePropertySpec {
    CodePoint(UnicodeProperty),
    String(UnicodeStringProperty),
}

struct Parser {
    chars: Vec<char>,
    i: usize,
    unicode_mode: bool,
    unicode_sets_mode: bool,
    capture_count: usize,
    named_captures: HashMap<String, usize>,
    named_capture_order: Vec<String>,
}

impl Parser {
    fn new(pattern: &str, unicode_mode: bool, unicode_sets_mode: bool) -> Self {
        Self {
            chars: pattern.chars().collect(),
            i: 0,
            unicode_mode,
            unicode_sets_mode,
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
        self.resolve_post_parse_escapes(&mut expr)?;
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
            '*' | '+' | '?' => Err(RegexError::new("nothing to repeat")),
            '{' => {
                if self.unicode_mode {
                    Err(RegexError::new("invalid quantifier syntax"))
                } else if self.looks_like_braced_quantifier(self.i - 1) {
                    Err(RegexError::new("nothing to repeat"))
                } else {
                    Ok(Expr::Literal('{'))
                }
            }
            '}' => {
                if self.unicode_mode {
                    Err(RegexError::new("invalid quantifier syntax"))
                } else {
                    Ok(Expr::Literal('}'))
                }
            }
            '\\' => self.parse_escape(false),
            _ => Ok(Expr::Literal(ch)),
        }
    }

    fn parse_group(&mut self) -> Result<Expr, RegexError> {
        if self.peek() == Some('?') {
            self.i += 1;
            let Some(ch) = self.next() else {
                return Err(RegexError::new("unsupported group syntax"));
            };
            match ch {
                ':' => {
                    let expr = self.parse_alternation()?;
                    self.expect(')')?;
                    Ok(Expr::Group {
                        index: None,
                        expr: Box::new(expr),
                    })
                }
                '=' => {
                    let expr = self.parse_alternation()?;
                    self.expect(')')?;
                    Ok(Expr::LookAhead {
                        positive: true,
                        expr: Box::new(expr),
                    })
                }
                '!' => {
                    let expr = self.parse_alternation()?;
                    self.expect(')')?;
                    Ok(Expr::LookAhead {
                        positive: false,
                        expr: Box::new(expr),
                    })
                }
                '<' => {
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
                'i' | 'm' | 's' | '-' => self.parse_flag_modifier_group(ch),
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

    fn parse_flag_modifier_group(&mut self, first: char) -> Result<Expr, RegexError> {
        let mut enabling = HashSet::new();
        let mut disabling = HashSet::new();
        let mut in_disabling = false;
        let mut current = first;

        loop {
            match current {
                'i' | 'm' | 's' => {
                    if in_disabling {
                        if !disabling.insert(current) || enabling.contains(&current) {
                            return Err(RegexError::new("duplicate flag in modifier group"));
                        }
                    } else if !enabling.insert(current) || disabling.contains(&current) {
                        return Err(RegexError::new("duplicate flag in modifier group"));
                    }
                }
                '-' => {
                    if in_disabling {
                        return Err(RegexError::new("invalid modifier group flags"));
                    }
                    in_disabling = true;
                }
                ':' => break,
                _ => return Err(RegexError::new("unsupported group syntax")),
            }

            let Some(next) = self.next() else {
                return Err(RegexError::new("unsupported group syntax"));
            };
            current = next;
        }

        if enabling.is_empty() && disabling.is_empty() {
            return Err(RegexError::new("invalid modifier group flags"));
        }

        let expr = self.parse_alternation()?;
        self.expect(')')?;

        Ok(Expr::FlagModifierGroup {
            expr: Box::new(expr),
            ignore_case: flag_modifier_value('i', &enabling, &disabling),
            multi_line: flag_modifier_value('m', &enabling, &disabling),
            dot_matches_new_line: flag_modifier_value('s', &enabling, &disabling),
        })
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
            '{' => {
                let checkpoint = self.i;
                match self.parse_braced_quantifier() {
                    Ok(quant) => Some(quant),
                    Err(err)
                        if !self.unicode_mode && err.message != "invalid quantifier range" =>
                    {
                        self.i = checkpoint;
                        None
                    }
                    Err(err) => return Err(err),
                }
            }
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

        let mut operands: Vec<Vec<ClassItem>> = vec![Vec::new()];
        let mut operators: Vec<ClassSetOperator> = Vec::new();

        while let Some(ch) = self.peek() {
            if ch == ']' {
                self.i += 1;
                if operators.is_empty() {
                    let items = operands
                        .pop()
                        .expect("character class always starts with one operand");
                    return self.build_class_expr_from_items(negated, items);
                }

                if operands.len() != operators.len() + 1 || operands.iter().any(Vec::is_empty) {
                    return Err(RegexError::new("invalid set operation"));
                }
                if operands
                    .iter()
                    .flatten()
                    .any(class_item_contains_unicode_string_property)
                {
                    return Err(RegexError::new("unsupported unicode set syntax"));
                }
                if negated
                    && operands
                        .iter()
                        .flatten()
                        .any(class_item_contains_non_scalar_string)
                {
                    return Err(RegexError::new("unsupported unicode set syntax"));
                }
                if operators
                    .windows(2)
                    .any(|pair| pair[0] != pair[1])
                {
                    return Err(RegexError::new("invalid set operation"));
                }
                if operands.iter().any(|operand| {
                    operand.len() != 1
                        || matches!(
                            operand.first(),
                            Some(ClassItem::Range(_, _)) | None
                        )
                }) {
                    return Err(RegexError::new("invalid set operation"));
                }
                let operand_classes = operands
                    .into_iter()
                    .map(|items| CharClass {
                        negated: false,
                        items,
                    })
                    .collect::<Vec<_>>();
                return Ok(Expr::ClassSetOperation {
                    operands: operand_classes,
                    operators,
                    negated,
                });
            }

            if self.unicode_sets_mode {
                if self.peek() == Some('&') && self.peek_next() == Some('&') {
                    if operands.last().map_or(true, Vec::is_empty) {
                        return Err(RegexError::new("invalid set operation"));
                    }
                    self.i += 2;
                    operators.push(ClassSetOperator::Intersection);
                    operands.push(Vec::new());
                    continue;
                }
                if self.peek() == Some('-') && self.peek_next() == Some('-') {
                    if operands.last().map_or(true, Vec::is_empty) {
                        return Err(RegexError::new("invalid set operation"));
                    }
                    self.i += 2;
                    operators.push(ClassSetOperator::Difference);
                    operands.push(Vec::new());
                    continue;
                }
            }

            let item = self.parse_class_item()?;
            if self.peek() == Some('-')
                && !(self.unicode_sets_mode && self.peek_next() == Some('-'))
            {
                let backup = self.i;
                self.i += 1;
                if self.peek() == Some(']') {
                    self.i = backup;
                    operands
                        .last_mut()
                        .expect("character class always has current operand")
                        .push(item);
                    continue;
                }
                let next_item = self.parse_class_item()?;
                match (item, next_item) {
                    (ClassItem::Char(start), ClassItem::Char(end)) => {
                        if (start as u32) > (end as u32) {
                            return Err(RegexError::new("invalid character class range"));
                        }
                        operands
                            .last_mut()
                            .expect("character class always has current operand")
                            .push(ClassItem::Range(start, end));
                    }
                    _ => {
                        return Err(RegexError::new(
                            "character class range bounds must be literal characters",
                        ));
                    }
                }
            } else {
                operands
                    .last_mut()
                    .expect("character class always has current operand")
                    .push(item);
            }
        }

        Err(RegexError::new("unterminated character class"))
    }

    fn parse_class_item(&mut self) -> Result<ClassItem, RegexError> {
        let Some(ch) = self.next() else {
            return Err(RegexError::new("unterminated character class"));
        };

        if ch != '\\' {
            if self.unicode_sets_mode && ch == '[' {
                let nested = self.parse_char_class()?;
                return Ok(ClassItem::NestedExpression(Box::new(nested)));
            }
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
                } else if self.unicode_mode {
                    Err(RegexError::new("invalid escape sequence"))
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
            '0' => {
                if self.peek().is_some_and(|next| next.is_ascii_digit()) {
                    if self.unicode_mode {
                        return Err(RegexError::new("invalid numeric backreference"));
                    }
                    if self.peek().is_some_and(|next| matches!(next, '0'..='7')) {
                        Ok(ClassItem::Char(self.parse_legacy_octal_escape_char('0')))
                    } else {
                        Ok(ClassItem::Char('\0'))
                    }
                } else {
                    Ok(ClassItem::Char('\0'))
                }
            }
            '1'..='9' => {
                if self.unicode_mode {
                    return Err(RegexError::new("invalid numeric backreference"));
                }
                if escaped == '8' || escaped == '9' {
                    Ok(ClassItem::Char(escaped))
                } else {
                    Ok(ClassItem::Char(self.parse_legacy_octal_escape_char(escaped)))
                }
            }
            'x' => {
                if self.peek_is_n_hex_digits(2) {
                    Ok(ClassItem::Char(self.parse_hex_char(2)?))
                } else if self.unicode_mode {
                    Err(RegexError::new("invalid hexadecimal escape sequence"))
                } else {
                    Ok(ClassItem::Char('x'))
                }
            }
            'u' => {
                if self.unicode_mode {
                    Ok(ClassItem::Char(self.parse_unicode_escape()?))
                } else if self.peek_is_n_hex_digits(4) {
                    Ok(ClassItem::Char(self.parse_hex_char(4)?))
                } else {
                    Ok(ClassItem::Char('u'))
                }
            }
            'p' | 'P' => {
                if !self.unicode_mode {
                    Ok(ClassItem::Char(escaped))
                } else {
                    match self.parse_unicode_property_escape()? {
                        UnicodePropertySpec::CodePoint(property) => Ok(ClassItem::UnicodeProperty {
                            property,
                            negated: escaped == 'P',
                        }),
                        UnicodePropertySpec::String(property) => {
                            if escaped == 'P' {
                                return Err(RegexError::new("invalid unicode property escape"));
                            }
                            Ok(ClassItem::UnicodeStringProperty(property))
                        }
                    }
                }
            }
            'q' => {
                if !self.unicode_sets_mode {
                    return Err(RegexError::new("invalid escape sequence"));
                }
                Ok(ClassItem::StringDisjunction(
                    self.parse_class_string_disjunction()?,
                ))
            }
            other => {
                if self.unicode_mode {
                    if is_unicode_identity_escape_in_class(other) {
                        Ok(ClassItem::Char(other))
                    } else {
                        Err(RegexError::new("invalid escape sequence"))
                    }
                } else {
                    Ok(ClassItem::Char(other))
                }
            }
        }
    }

    fn parse_class_string_disjunction(&mut self) -> Result<Vec<String>, RegexError> {
        if self.next() != Some('{') {
            return Err(RegexError::new("invalid escape sequence"));
        }

        let mut alternatives = vec![String::new()];
        loop {
            let Some(ch) = self.peek() else {
                return Err(RegexError::new("unterminated class string disjunction"));
            };

            if ch == '}' {
                self.i += 1;
                return Ok(alternatives);
            }

            if ch == '|' {
                self.i += 1;
                alternatives.push(String::new());
                continue;
            }

            let value = if ch == '\\' {
                self.i += 1;
                self.parse_class_string_escape()?
            } else {
                self.i += 1;
                ch
            };
            alternatives
                .last_mut()
                .expect("class string disjunction always has one alternative")
                .push(value);
        }
    }

    fn parse_class_string_escape(&mut self) -> Result<char, RegexError> {
        let Some(escaped) = self.next() else {
            return Err(RegexError::new("unterminated escape sequence"));
        };

        match escaped {
            'c' => {
                if self.peek().is_some_and(|next| next.is_ascii_alphabetic()) {
                    let next = self.next().expect("peek guaranteed a character");
                    Ok(control_escape_char(next))
                } else {
                    Err(RegexError::new("invalid escape sequence"))
                }
            }
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            'v' => Ok('\u{000B}'),
            'f' => Ok('\u{000C}'),
            'b' => Ok('\u{0008}'),
            '0' => {
                if self.peek().is_some_and(|next| next.is_ascii_digit()) {
                    Err(RegexError::new("invalid decimal escape"))
                } else {
                    Ok('\0')
                }
            }
            '1'..='9' => Err(RegexError::new("invalid decimal escape")),
            'x' => {
                if self.peek_is_n_hex_digits(2) {
                    self.parse_hex_char(2)
                } else {
                    Err(RegexError::new("invalid hexadecimal escape sequence"))
                }
            }
            'u' => self.parse_unicode_escape(),
            other => {
                if is_class_string_identity_escape(other) {
                    Ok(other)
                } else {
                    Err(RegexError::new("invalid escape sequence"))
                }
            }
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
                } else if self.unicode_mode {
                    Err(RegexError::new("invalid escape sequence"))
                } else {
                    Ok(Expr::Literal('c'))
                }
            }
            '0' => {
                if self.peek().is_some_and(|next| next.is_ascii_digit()) {
                    if self.unicode_mode {
                        return Err(RegexError::new("invalid numeric backreference"));
                    }
                    if self.peek().is_some_and(|next| matches!(next, '0'..='7')) {
                        Ok(Expr::Literal(self.parse_legacy_octal_escape_char('0')))
                    } else {
                        Ok(Expr::Literal('\0'))
                    }
                } else {
                    Ok(Expr::Literal('\0'))
                }
            }
            'x' => {
                if self.peek_is_n_hex_digits(2) {
                    Ok(Expr::Literal(self.parse_hex_char(2)?))
                } else if self.unicode_mode {
                    Err(RegexError::new("invalid hexadecimal escape sequence"))
                } else {
                    Ok(Expr::Literal('x'))
                }
            }
            'u' => {
                if self.unicode_mode {
                    Ok(Expr::Literal(self.parse_unicode_escape()?))
                } else if self.peek_is_n_hex_digits(4) {
                    Ok(Expr::Literal(self.parse_hex_char(4)?))
                } else {
                    Ok(Expr::Literal('u'))
                }
            }
            'p' | 'P' => {
                if !self.unicode_mode {
                    Ok(Expr::Literal(ch))
                } else {
                    match self.parse_unicode_property_escape()? {
                        UnicodePropertySpec::CodePoint(property) => {
                            Ok(Expr::CharClass(CharClass {
                                negated: false,
                                items: vec![ClassItem::UnicodeProperty {
                                    property,
                                    negated: ch == 'P',
                                }],
                            }))
                        }
                        UnicodePropertySpec::String(property) => {
                            if ch == 'P' {
                                return Err(RegexError::new("invalid unicode property escape"));
                            }
                            Ok(Expr::UnicodeStringProperty(property))
                        }
                    }
                }
            }
            'k' => {
                if self.peek() != Some('<') {
                    if self.unicode_mode {
                        return Err(RegexError::new("invalid named backreference syntax"));
                    }
                    return Ok(Expr::Literal('k'));
                }
                self.i += 1; // consume '<'
                let name = match self.parse_raw_group_name_until_gt() {
                    Some(name) => name,
                    None => {
                        if self.unicode_mode {
                            return Err(RegexError::new("invalid named backreference syntax"));
                        }
                        return Ok(Expr::Sequence(vec![Expr::Literal('k'), Expr::Literal('<')]));
                    }
                };
                Ok(Expr::NamedBackReference(name))
            }
            '1'..='9' => self.parse_numeric_backreference(ch),
            other => {
                if self.unicode_mode {
                    if is_unicode_identity_escape_outside_class(other) {
                        Ok(Expr::Literal(other))
                    } else {
                        Err(RegexError::new("invalid escape sequence"))
                    }
                } else {
                    Ok(Expr::Literal(other))
                }
            }
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

    fn parse_raw_group_name_until_gt(&mut self) -> Option<String> {
        let start = self.i;
        while let Some(ch) = self.peek() {
            if ch == '>' {
                let name = self.chars[start..self.i].iter().collect::<String>();
                self.i += 1; // consume '>'
                return Some(name);
            }
            self.i += 1;
        }
        None
    }

    fn parse_numeric_backreference(&mut self, first_digit: char) -> Result<Expr, RegexError> {
        let mut raw = String::new();
        raw.push(first_digit);
        while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            raw.push(self.next().expect("peek guaranteed a digit"));
        }
        Ok(Expr::DecimalEscape(raw))
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

    fn parse_unicode_property_escape(&mut self) -> Result<UnicodePropertySpec, RegexError> {
        if self.next() != Some('{') {
            return Err(RegexError::new("invalid unicode property escape"));
        }
        let start = self.i;
        while let Some(ch) = self.peek() {
            if ch == '}' {
                break;
            }
            self.i += 1;
        }
        if self.peek() != Some('}') || self.i == start {
            return Err(RegexError::new("invalid unicode property escape"));
        }
        let raw: String = self.chars[start..self.i].iter().collect();
        self.i += 1; // consume '}'
        parse_unicode_property_name(&raw, self.unicode_sets_mode)
            .ok_or_else(|| RegexError::new("invalid unicode property escape"))
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

    fn peek_is_n_hex_digits(&self, digits: usize) -> bool {
        let start = self.i;
        let end = start.saturating_add(digits);
        end <= self.chars.len()
            && self.chars[start..end]
                .iter()
                .all(|ch| ch.is_ascii_hexdigit())
    }

    fn build_class_expr_from_items(
        &self,
        negated: bool,
        items: Vec<ClassItem>,
    ) -> Result<Expr, RegexError> {
        let mut branches = Vec::new();
        let mut string_branches = Vec::new();
        let mut scalar_items = Vec::new();

        for item in items {
            match item {
                ClassItem::UnicodeStringProperty(property) => {
                    if negated {
                        return Err(RegexError::new("unsupported unicode set syntax"));
                    }
                    branches.push(Expr::UnicodeStringProperty(property));
                }
                ClassItem::StringDisjunction(alternatives) => {
                    for alternative in alternatives {
                        if let Some(ch) = single_char_string(&alternative) {
                            scalar_items.push(ClassItem::Char(ch));
                        } else {
                            if negated {
                                return Err(RegexError::new("unsupported unicode set syntax"));
                            }
                            string_branches.push(alternative);
                        }
                    }
                }
                ClassItem::NestedExpression(expr) => {
                    if expr_contains_non_scalar_string(&expr)
                        || expr_contains_unicode_string_property(&expr)
                    {
                        if negated {
                            return Err(RegexError::new("unsupported unicode set syntax"));
                        }
                        branches.push(*expr);
                    } else {
                        scalar_items.push(ClassItem::NestedExpression(expr));
                    }
                }
                other => scalar_items.push(other),
            }
        }

        string_branches.sort_by(|left, right| right.chars().count().cmp(&left.chars().count()));
        for literal in string_branches {
            branches.push(expr_from_literal_string(&literal));
        }

        if branches.is_empty() {
            return Ok(Expr::CharClass(CharClass {
                negated,
                items: scalar_items,
            }));
        }
        if !scalar_items.is_empty() {
            branches.push(Expr::CharClass(CharClass {
                negated: false,
                items: scalar_items,
            }));
        }
        if branches.len() == 1 {
            Ok(branches.pop().expect("branches has one element"))
        } else {
            Ok(Expr::Alternation(branches))
        }
    }

    fn expect(&mut self, ch: char) -> Result<(), RegexError> {
        if self.next() == Some(ch) {
            Ok(())
        } else {
            Err(RegexError::new("unterminated group or invalid syntax"))
        }
    }

    fn resolve_post_parse_escapes(&self, expr: &mut Expr) -> Result<(), RegexError> {
        match expr {
            Expr::NamedBackReference(name) => {
                let has_named_captures = !self.named_captures.is_empty();
                if !self.unicode_mode && !has_named_captures {
                    *expr = Self::literal_named_backreference_expr(name);
                    return Ok(());
                }

                if !Self::is_valid_group_name(name) {
                    return Err(RegexError::new("invalid capture group name"));
                }

                let Some(index) = self.named_captures.get(name).copied() else {
                    return Err(RegexError::new(format!(
                        "unknown named backreference: {name}"
                    )));
                };
                *expr = Expr::BackReference(index);
                Ok(())
            }
            Expr::DecimalEscape(raw) => {
                *expr = self.resolve_decimal_escape(raw)?;
                Ok(())
            }
            Expr::Sequence(parts) | Expr::Alternation(parts) => {
                for part in parts {
                    self.resolve_post_parse_escapes(part)?;
                }
                Ok(())
            }
            Expr::Group { expr, .. }
            | Expr::FlagModifierGroup { expr, .. }
            | Expr::LookAhead { expr, .. }
            | Expr::LookBehind { expr, .. }
            | Expr::Quantifier { expr, .. } => self.resolve_post_parse_escapes(expr),
            _ => Ok(()),
        }
    }

    fn resolve_decimal_escape(&self, raw: &str) -> Result<Expr, RegexError> {
        let index = raw.parse::<usize>().ok();
        if self.unicode_mode {
            let Some(index) = index else {
                return Err(RegexError::new("invalid numeric backreference"));
            };
            if index == 0 || index > self.capture_count {
                return Err(RegexError::new("invalid numeric backreference"));
            }
            return Ok(Expr::BackReference(index));
        }

        if let Some(index) = index {
            if index > 0 && index <= self.capture_count {
                return Ok(Expr::BackReference(index));
            }
        }

        Ok(Self::legacy_decimal_escape_expr(raw))
    }

    fn legacy_decimal_escape_expr(raw: &str) -> Expr {
        let chars = raw.chars().collect::<Vec<_>>();
        let Some(first) = chars.first().copied() else {
            return Expr::Empty;
        };

        if first == '8' || first == '9' {
            return Self::literal_sequence_from_chars(&chars);
        }

        let mut octal = String::new();
        octal.push(first);
        let mut used = 1usize;
        while used < chars.len() && used < 3 && matches!(chars[used], '0'..='7') {
            octal.push(chars[used]);
            used += 1;
        }
        let value = u32::from_str_radix(&octal, 8).expect("legacy octal digits are valid");
        let first_expr = char::from_u32(value)
            .map(Expr::Literal)
            .unwrap_or(Expr::Literal('\0'));
        if used >= chars.len() {
            first_expr
        } else {
            let mut parts = vec![first_expr];
            parts.extend(chars[used..].iter().copied().map(Expr::Literal));
            Expr::Sequence(parts)
        }
    }

    fn literal_sequence_from_chars(chars: &[char]) -> Expr {
        if chars.len() == 1 {
            Expr::Literal(chars[0])
        } else {
            Expr::Sequence(chars.iter().copied().map(Expr::Literal).collect::<Vec<_>>())
        }
    }

    fn parse_legacy_octal_escape_char(&mut self, first: char) -> char {
        let mut raw = String::new();
        raw.push(first);
        for _ in 0..2 {
            if self.peek().is_some_and(|ch| matches!(ch, '0'..='7')) {
                raw.push(self.next().expect("peek guaranteed a digit"));
            } else {
                break;
            }
        }
        let value = u32::from_str_radix(&raw, 8).expect("legacy octal digits are valid");
        char::from_u32(value).unwrap_or('\0')
    }

    fn literal_named_backreference_expr(name: &str) -> Expr {
        let mut chars = Vec::with_capacity(name.chars().count() + 3);
        chars.push('k');
        chars.push('<');
        chars.extend(name.chars());
        chars.push('>');
        Self::literal_sequence_from_chars(&chars)
    }

    fn is_valid_group_name(name: &str) -> bool {
        !name.is_empty() && name.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    }

    fn looks_like_braced_quantifier(&self, brace_index: usize) -> bool {
        if self.chars.get(brace_index) != Some(&'{') {
            return false;
        }

        let mut idx = brace_index + 1;
        let mut saw_digit = false;
        while self.chars.get(idx).is_some_and(|ch| ch.is_ascii_digit()) {
            saw_digit = true;
            idx += 1;
        }
        if !saw_digit {
            return false;
        }

        match self.chars.get(idx).copied() {
            Some('}') => true,
            Some(',') => {
                idx += 1;
                while self.chars.get(idx).is_some_and(|ch| ch.is_ascii_digit()) {
                    idx += 1;
                }
                self.chars.get(idx) == Some(&'}')
            }
            _ => false,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.i + 1).copied()
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

fn class_set_operation_matches_char(
    operands: &[CharClass],
    operators: &[ClassSetOperator],
    negated: bool,
    ch: char,
    ignore_case: bool,
) -> bool {
    let Some(first) = operands.first() else {
        return false;
    };
    let mut matched = first.matches(ch, ignore_case);
    for (op, operand) in operators.iter().zip(operands.iter().skip(1)) {
        let right = operand.matches(ch, ignore_case);
        match op {
            ClassSetOperator::Intersection => matched = matched && right,
            ClassSetOperator::Difference => matched = matched && !right,
        }
    }
    if negated { !matched } else { matched }
}

fn class_set_operation_match_ends(
    prepared: &PreparedInput<'_>,
    pos: usize,
    operands: &[CharClass],
    operators: &[ClassSetOperator],
    negated: bool,
    ignore_case: bool,
) -> Vec<usize> {
    let Some(first) = operands.first() else {
        return Vec::new();
    };

    let mut current = first
        .match_ends(prepared, pos, ignore_case)
        .into_iter()
        .collect::<HashSet<_>>();
    for (op, operand) in operators.iter().zip(operands.iter().skip(1)) {
        let rhs = operand
            .match_ends(prepared, pos, ignore_case)
            .into_iter()
            .collect::<HashSet<_>>();
        match op {
            ClassSetOperator::Intersection => current.retain(|end| rhs.contains(end)),
            ClassSetOperator::Difference => current.retain(|end| !rhs.contains(end)),
        }
    }

    if negated {
        let Some(_) = prepared.char_at(pos) else {
            return Vec::new();
        };
        let scalar_end = pos + 1;
        if current.contains(&scalar_end) {
            Vec::new()
        } else {
            vec![scalar_end]
        }
    } else {
        dedupe_sorted_desc(current.into_iter().collect::<Vec<_>>())
    }
}

fn expr_match_ends(
    expr: &Expr,
    prepared: &PreparedInput<'_>,
    pos: usize,
    ignore_case: bool,
) -> Vec<usize> {
    match expr {
        Expr::Empty => vec![pos],
        Expr::Literal(expected) => prepared
            .char_at(pos)
            .filter(|actual| chars_equal(*actual, *expected, ignore_case))
            .map(|_| pos + 1)
            .into_iter()
            .collect::<Vec<_>>(),
        Expr::Sequence(parts) => {
            let mut cursors = vec![pos];
            for part in parts {
                let mut next = Vec::new();
                for cursor in &cursors {
                    next.extend(expr_match_ends(part, prepared, *cursor, ignore_case));
                }
                if next.is_empty() {
                    return Vec::new();
                }
                cursors = dedupe_sorted_desc(next);
            }
            cursors
        }
        Expr::Alternation(branches) => {
            let mut out = Vec::new();
            for branch in branches {
                out.extend(expr_match_ends(branch, prepared, pos, ignore_case));
            }
            dedupe_sorted_desc(out)
        }
        Expr::CharClass(class) => class.match_ends(prepared, pos, ignore_case),
        Expr::ClassSetOperation {
            operands,
            operators,
            negated,
        } => class_set_operation_match_ends(
            prepared,
            pos,
            operands,
            operators,
            *negated,
            ignore_case,
        ),
        Expr::UnicodeStringProperty(property) => property
            .match_at(prepared, pos)
            .into_iter()
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    }
}

fn expr_matches_single_char(expr: &Expr, ch: char, ignore_case: bool) -> bool {
    match expr {
        Expr::CharClass(class) => class.matches(ch, ignore_case),
        Expr::ClassSetOperation {
            operands,
            operators,
            negated,
        } => class_set_operation_matches_char(operands, operators, *negated, ch, ignore_case),
        Expr::Alternation(branches) => branches
            .iter()
            .any(|branch| expr_matches_single_char(branch, ch, ignore_case)),
        Expr::Literal(expected) => chars_equal(ch, *expected, ignore_case),
        Expr::UnicodeStringProperty(_) => false,
        _ => false,
    }
}

fn single_char_string(value: &str) -> Option<char> {
    let mut chars = value.chars();
    let first = chars.next()?;
    if chars.next().is_none() {
        Some(first)
    } else {
        None
    }
}

fn expr_from_literal_string(value: &str) -> Expr {
    let chars = value.chars().collect::<Vec<_>>();
    if chars.is_empty() {
        Expr::Empty
    } else if chars.len() == 1 {
        Expr::Literal(chars[0])
    } else {
        Expr::Sequence(chars.into_iter().map(Expr::Literal).collect::<Vec<_>>())
    }
}

fn match_literal_string_at(
    prepared: &PreparedInput<'_>,
    pos: usize,
    candidate: &str,
    ignore_case: bool,
) -> Option<usize> {
    let mut index = pos;
    for expected in candidate.chars() {
        let actual = prepared.char_at(index)?;
        if !chars_equal(actual, expected, ignore_case) {
            return None;
        }
        index += 1;
    }
    Some(index)
}

fn dedupe_sorted_desc(mut values: Vec<usize>) -> Vec<usize> {
    values.sort_unstable();
    values.dedup();
    values.reverse();
    values
}

fn class_item_contains_non_scalar_string(item: &ClassItem) -> bool {
    match item {
        ClassItem::UnicodeStringProperty(_) => false,
        ClassItem::StringDisjunction(alternatives) => alternatives
            .iter()
            .any(|alternative| single_char_string(alternative).is_none()),
        ClassItem::NestedExpression(expr) => expr_contains_non_scalar_string(expr),
        _ => false,
    }
}

fn expr_contains_non_scalar_string(expr: &Expr) -> bool {
    match expr {
        Expr::Empty => true,
        Expr::Literal(_) => false,
        Expr::UnicodeStringProperty(_) => true,
        Expr::CharClass(class) => class
            .items
            .iter()
            .any(class_item_contains_non_scalar_string),
        Expr::ClassSetOperation { operands, .. } => operands.iter().any(|operand| {
            operand
                .items
                .iter()
                .any(class_item_contains_non_scalar_string)
        }),
        Expr::Sequence(parts) => {
            if parts.len() != 1 {
                true
            } else {
                parts.iter().any(expr_contains_non_scalar_string)
            }
        }
        Expr::Alternation(branches) => branches.iter().any(expr_contains_non_scalar_string),
        Expr::Group { expr, .. }
        | Expr::FlagModifierGroup { expr, .. }
        | Expr::LookAhead { expr, .. }
        | Expr::LookBehind { expr, .. }
        | Expr::Quantifier { expr, .. } => expr_contains_non_scalar_string(expr),
        _ => false,
    }
}

fn class_item_contains_unicode_string_property(item: &ClassItem) -> bool {
    match item {
        ClassItem::UnicodeStringProperty(_) => true,
        ClassItem::NestedExpression(expr) => expr_contains_unicode_string_property(expr),
        _ => false,
    }
}

fn expr_contains_unicode_string_property(expr: &Expr) -> bool {
    match expr {
        Expr::UnicodeStringProperty(_) => true,
        Expr::CharClass(class) => class
            .items
            .iter()
            .any(class_item_contains_unicode_string_property),
        Expr::ClassSetOperation { operands, .. } => operands.iter().any(|operand| {
            operand
                .items
                .iter()
                .any(class_item_contains_unicode_string_property)
        }),
        Expr::Alternation(branches) | Expr::Sequence(branches) => branches
            .iter()
            .any(expr_contains_unicode_string_property),
        Expr::Group { expr, .. }
        | Expr::FlagModifierGroup { expr, .. }
        | Expr::LookAhead { expr, .. }
        | Expr::LookBehind { expr, .. }
        | Expr::Quantifier { expr, .. } => expr_contains_unicode_string_property(expr),
        _ => false,
    }
}

fn is_class_string_identity_escape(ch: char) -> bool {
    ch.is_ascii_punctuation() && ch != '_'
}

fn flag_modifier_value(
    flag: char,
    enabling: &HashSet<char>,
    disabling: &HashSet<char>,
) -> Option<bool> {
    if enabling.contains(&flag) {
        Some(true)
    } else if disabling.contains(&flag) {
        Some(false)
    } else {
        None
    }
}

fn parse_unicode_property_name(raw: &str, unicode_sets_mode: bool) -> Option<UnicodePropertySpec> {
    if let Some((name, value)) = raw.split_once('=') {
        return parse_unicode_property_pair(name, value);
    }
    match raw {
        "L" | "Letter" => Some(UnicodePropertySpec::CodePoint(UnicodeProperty::Letter)),
        "N" | "Number" => Some(UnicodePropertySpec::CodePoint(UnicodeProperty::Number)),
        "Nd" | "Decimal_Number" => Some(UnicodePropertySpec::CodePoint(UnicodeProperty::DecimalNumber)),
        "ASCII" => Some(UnicodePropertySpec::CodePoint(UnicodeProperty::Ascii)),
        "Any" => Some(UnicodePropertySpec::CodePoint(UnicodeProperty::Any)),
        "RGI_Emoji" if unicode_sets_mode => {
            Some(UnicodePropertySpec::String(UnicodeStringProperty::RgiEmoji))
        }
        _ => None,
    }
}

fn parse_unicode_property_pair(name: &str, value: &str) -> Option<UnicodePropertySpec> {
    match name {
        "gc" | "General_Category" => match value {
            "L" | "Letter" => Some(UnicodePropertySpec::CodePoint(UnicodeProperty::Letter)),
            "N" | "Number" => Some(UnicodePropertySpec::CodePoint(UnicodeProperty::Number)),
            "Nd" | "Decimal_Number" => Some(UnicodePropertySpec::CodePoint(UnicodeProperty::DecimalNumber)),
            _ => None,
        },
        "sc" | "Script" | "scx" | "Script_Extensions" => match value {
            "Latin" => Some(UnicodePropertySpec::CodePoint(UnicodeProperty::ScriptLatin)),
            "Greek" => Some(UnicodePropertySpec::CodePoint(UnicodeProperty::ScriptGreek)),
            _ => None,
        },
        _ => None,
    }
}

fn match_rgi_emoji_at(prepared: &PreparedInput<'_>, pos: usize) -> Option<usize> {
    let ch = prepared.char_at(pos)?;

    // Keycap emoji sequences, e.g. 0️⃣, #️⃣, *️⃣.
    if is_keycap_base(ch) {
        let mut idx = pos + 1;
        if prepared.char_at(idx) == Some('\u{FE0F}') {
            idx += 1;
        }
        if prepared.char_at(idx) == Some('\u{20E3}') {
            return Some(idx + 1);
        }
    }

    // Flag emoji are two regional indicators.
    if is_regional_indicator(ch) && prepared.char_at(pos + 1).is_some_and(is_regional_indicator) {
        return Some(pos + 2);
    }

    let mut idx = match_emoji_atom(prepared, pos)?;
    while prepared.char_at(idx) == Some('\u{200D}') {
        let next = match_emoji_atom(prepared, idx + 1)?;
        idx = next;
    }
    Some(idx)
}

fn match_emoji_atom(prepared: &PreparedInput<'_>, pos: usize) -> Option<usize> {
    let ch = prepared.char_at(pos)?;
    if !is_emoji_base(ch) {
        return None;
    }

    let mut idx = pos + 1;
    if prepared.char_at(idx) == Some('\u{FE0F}') {
        idx += 1;
    }
    if prepared
        .char_at(idx)
        .is_some_and(|modifier| (0x1F3FB..=0x1F3FF).contains(&(modifier as u32)))
    {
        idx += 1;
    }
    Some(idx)
}

fn is_keycap_base(ch: char) -> bool {
    ch.is_ascii_digit() || matches!(ch, '#' | '*')
}

fn is_regional_indicator(ch: char) -> bool {
    (0x1F1E6..=0x1F1FF).contains(&(ch as u32))
}

fn is_emoji_base(ch: char) -> bool {
    let code = ch as u32;
    matches!(
        code,
        0x1F300..=0x1FAFF
            | 0x2600..=0x27BF
            | 0x2300..=0x23FF
            | 0x2194..=0x21AA
            | 0x00A9
            | 0x00AE
            | 0x203C
            | 0x2049
            | 0x2122
            | 0x2139
            | 0x2934
            | 0x2935
            | 0x3030
            | 0x303D
            | 0x3297
            | 0x3299
    )
}

fn control_escape_char(letter: char) -> char {
    let upper = letter.to_ascii_uppercase() as u8;
    char::from(upper - b'@')
}

fn is_unicode_identity_escape_outside_class(ch: char) -> bool {
    matches!(
        ch,
        '$'
            | '('
            | ')'
            | '*'
            | '+'
            | '.'
            | '/'
            | '?'
            | '['
            | '\\'
            | ']'
            | '^'
            | '{'
            | '|'
            | '}'
    )
}

fn is_unicode_identity_escape_in_class(ch: char) -> bool {
    matches!(
        ch,
        '$'
            | '('
            | ')'
            | '*'
            | '+'
            | '-'
            | '.'
            | '/'
            | '?'
            | '['
            | '\\'
            | ']'
            | '^'
            | '{'
            | '|'
            | '}'
    )
}

fn char_in_ranges(ch: char, ranges: &[(u32, u32)]) -> bool {
    let code = ch as u32;
    ranges
        .iter()
        .any(|&(start, end)| start <= code && code <= end)
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

const UNICODE_DECIMAL_NUMBER_RANGES: &[(u32, u32)] = &[
    (0x30, 0x39),
    (0x660, 0x669),
    (0x6F0, 0x6F9),
    (0x7C0, 0x7C9),
    (0x966, 0x96F),
    (0x9E6, 0x9EF),
    (0xA66, 0xA6F),
    (0xAE6, 0xAEF),
    (0xB66, 0xB6F),
    (0xBE6, 0xBEF),
    (0xC66, 0xC6F),
    (0xCE6, 0xCEF),
    (0xD66, 0xD6F),
    (0xDE6, 0xDEF),
    (0xE50, 0xE59),
    (0xED0, 0xED9),
    (0xF20, 0xF29),
    (0x1040, 0x1049),
    (0x1090, 0x1099),
    (0x17E0, 0x17E9),
    (0x1810, 0x1819),
    (0x1946, 0x194F),
    (0x19D0, 0x19D9),
    (0x1A80, 0x1A89),
    (0x1A90, 0x1A99),
    (0x1B50, 0x1B59),
    (0x1BB0, 0x1BB9),
    (0x1C40, 0x1C49),
    (0x1C50, 0x1C59),
    (0xA620, 0xA629),
    (0xA8D0, 0xA8D9),
    (0xA900, 0xA909),
    (0xA9D0, 0xA9D9),
    (0xA9F0, 0xA9F9),
    (0xAA50, 0xAA59),
    (0xABF0, 0xABF9),
    (0xFF10, 0xFF19),
    (0x104A0, 0x104A9),
    (0x10D30, 0x10D39),
    (0x10D40, 0x10D49),
    (0x11066, 0x1106F),
    (0x110F0, 0x110F9),
    (0x11136, 0x1113F),
    (0x111D0, 0x111D9),
    (0x112F0, 0x112F9),
    (0x11450, 0x11459),
    (0x114D0, 0x114D9),
    (0x11650, 0x11659),
    (0x116C0, 0x116C9),
    (0x116D0, 0x116E3),
    (0x11730, 0x11739),
    (0x118E0, 0x118E9),
    (0x11950, 0x11959),
    (0x11BF0, 0x11BF9),
    (0x11C50, 0x11C59),
    (0x11D50, 0x11D59),
    (0x11DA0, 0x11DA9),
    (0x11DE0, 0x11DE9),
    (0x11F50, 0x11F59),
    (0x16130, 0x16139),
    (0x16A60, 0x16A69),
    (0x16AC0, 0x16AC9),
    (0x16B50, 0x16B59),
    (0x16D70, 0x16D79),
    (0x1CCF0, 0x1CCF9),
    (0x1D7CE, 0x1D7FF),
    (0x1E140, 0x1E149),
    (0x1E2F0, 0x1E2F9),
    (0x1E4F0, 0x1E4F9),
    (0x1E5F1, 0x1E5FA),
    (0x1E950, 0x1E959),
    (0x1FBF0, 0x1FBF9),
];

const UNICODE_SCRIPT_LATIN_RANGES: &[(u32, u32)] = &[
    (0x41, 0x5A),
    (0x61, 0x7A),
    (0xAA, 0xAA),
    (0xBA, 0xBA),
    (0xC0, 0xD6),
    (0xD8, 0xF6),
    (0xF8, 0x2B8),
    (0x2E0, 0x2E4),
    (0x1D00, 0x1D25),
    (0x1D2C, 0x1D5C),
    (0x1D62, 0x1D65),
    (0x1D6B, 0x1D77),
    (0x1D79, 0x1DBE),
    (0x1E00, 0x1EFF),
    (0x2071, 0x2071),
    (0x207F, 0x207F),
    (0x2090, 0x209C),
    (0x212A, 0x212B),
    (0x2132, 0x2132),
    (0x214E, 0x214E),
    (0x2160, 0x2188),
    (0x2C60, 0x2C7F),
    (0xA722, 0xA787),
    (0xA78B, 0xA7DC),
    (0xA7F1, 0xA7FF),
    (0xAB30, 0xAB5A),
    (0xAB5C, 0xAB64),
    (0xAB66, 0xAB69),
    (0xFB00, 0xFB06),
    (0xFF21, 0xFF3A),
    (0xFF41, 0xFF5A),
    (0x10780, 0x10785),
    (0x10787, 0x107B0),
    (0x107B2, 0x107BA),
    (0x1DF00, 0x1DF1E),
    (0x1DF25, 0x1DF2A),
];

const UNICODE_SCRIPT_GREEK_RANGES: &[(u32, u32)] = &[
    (0x370, 0x373),
    (0x375, 0x377),
    (0x37A, 0x37D),
    (0x37F, 0x37F),
    (0x384, 0x384),
    (0x386, 0x386),
    (0x388, 0x38A),
    (0x38C, 0x38C),
    (0x38E, 0x3A1),
    (0x3A3, 0x3E1),
    (0x3F0, 0x3FF),
    (0x1D26, 0x1D2A),
    (0x1D5D, 0x1D61),
    (0x1D66, 0x1D6A),
    (0x1DBF, 0x1DBF),
    (0x1F00, 0x1F15),
    (0x1F18, 0x1F1D),
    (0x1F20, 0x1F45),
    (0x1F48, 0x1F4D),
    (0x1F50, 0x1F57),
    (0x1F59, 0x1F59),
    (0x1F5B, 0x1F5B),
    (0x1F5D, 0x1F5D),
    (0x1F5F, 0x1F7D),
    (0x1F80, 0x1FB4),
    (0x1FB6, 0x1FC4),
    (0x1FC6, 0x1FD3),
    (0x1FD6, 0x1FDB),
    (0x1FDD, 0x1FEF),
    (0x1FF2, 0x1FF4),
    (0x1FF6, 0x1FFE),
    (0x2126, 0x2126),
    (0xAB65, 0xAB65),
    (0x10140, 0x1018E),
    (0x101A0, 0x101A0),
    (0x1D200, 0x1D245),
];
