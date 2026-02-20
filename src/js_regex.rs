use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Clone)]
pub(crate) struct Regex {
    backend: fancy_regex::Regex,
}

impl Regex {
    pub(crate) fn new(pattern: &str) -> Result<Self, RegexError> {
        let backend = fancy_regex::Regex::new(pattern).map_err(RegexError::from)?;
        Ok(Self { backend })
    }

    pub(crate) fn is_match(&self, input: &str) -> Result<bool, RegexError> {
        self.backend.is_match(input).map_err(RegexError::from)
    }

    pub(crate) fn captures(&self, input: &str) -> Result<Option<Captures>, RegexError> {
        let captures = self.backend.captures(input).map_err(RegexError::from)?;
        Ok(captures.as_ref().map(Captures::from_backend))
    }

    pub(crate) fn captures_from_pos(
        &self,
        input: &str,
        start: usize,
    ) -> Result<Option<Captures>, RegexError> {
        let captures = self
            .backend
            .captures_from_pos(input, start)
            .map_err(RegexError::from)?;
        Ok(captures.as_ref().map(Captures::from_backend))
    }

    pub(crate) fn captures_all(&self, input: &str) -> Result<Vec<Captures>, RegexError> {
        let mut out = Vec::new();
        for captures in self.backend.captures_iter(input) {
            let captures = captures.map_err(RegexError::from)?;
            out.push(Captures::from_backend(&captures));
        }
        Ok(out)
    }

    pub(crate) fn find_all(&self, input: &str) -> Result<Vec<Match>, RegexError> {
        let mut out = Vec::new();
        for matched in self.backend.find_iter(input) {
            let matched = matched.map_err(RegexError::from)?;
            out.push(Match::from_backend(matched));
        }
        Ok(out)
    }

    pub(crate) fn find(&self, input: &str) -> Result<Option<Match>, RegexError> {
        let matched = self.backend.find(input).map_err(RegexError::from)?;
        Ok(matched.map(Match::from_backend))
    }

    pub(crate) fn split_all(&self, input: &str) -> Result<Vec<String>, RegexError> {
        let mut out = Vec::new();
        for part in self.backend.split(input) {
            out.push(part.map_err(RegexError::from)?.to_string());
        }
        Ok(out)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RegexBuilder {
    pattern: String,
    case_insensitive: bool,
    multi_line: bool,
    dot_matches_new_line: bool,
}

impl RegexBuilder {
    pub(crate) fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            case_insensitive: false,
            multi_line: false,
            dot_matches_new_line: false,
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

    pub(crate) fn build(&self) -> Result<Regex, RegexError> {
        let mut builder = fancy_regex::RegexBuilder::new(&self.pattern);
        builder.case_insensitive(self.case_insensitive);
        builder.multi_line(self.multi_line);
        builder.dot_matches_new_line(self.dot_matches_new_line);
        let backend = builder.build().map_err(RegexError::from)?;
        Ok(Regex { backend })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Captures {
    groups: Vec<Option<Match>>,
}

impl Captures {
    fn from_backend(captures: &fancy_regex::Captures<'_>) -> Self {
        let mut groups = Vec::with_capacity(captures.len());
        for idx in 0..captures.len() {
            let matched = captures.get(idx).map(Match::from_backend);
            groups.push(matched);
        }
        Self { groups }
    }

    pub(crate) fn len(&self) -> usize {
        self.groups.len()
    }

    pub(crate) fn get(&self, index: usize) -> Option<&Match> {
        self.groups.get(index).and_then(Option::as_ref)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Match {
    start: usize,
    end: usize,
    text: String,
}

impl Match {
    fn from_backend(matched: fancy_regex::Match<'_>) -> Self {
        Self {
            start: matched.start(),
            end: matched.end(),
            text: matched.as_str().to_string(),
        }
    }

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

impl fmt::Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RegexError {}

impl From<fancy_regex::Error> for RegexError {
    fn from(value: fancy_regex::Error) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

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

fn is_regex_meta(ch: char) -> bool {
    matches!(
        ch,
        '\\' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '^' | '$' | '/'
    )
}
