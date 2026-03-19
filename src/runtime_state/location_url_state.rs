use super::*;
use idna::domain_to_ascii;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocationParts {
    pub(crate) scheme: String,
    pub(crate) has_authority: bool,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) hostname: String,
    pub(crate) port: String,
    pub(crate) pathname: String,
    pub(crate) opaque_path: String,
    pub(crate) search: String,
    pub(crate) hash: String,
}

impl LocationParts {
    pub(crate) fn protocol(&self) -> String {
        format!("{}:", self.scheme)
    }

    pub(crate) fn effective_port(&self) -> String {
        if self.has_authority
            && default_port_for_scheme(&self.scheme)
                .is_some_and(|default_port| self.port == default_port)
        {
            String::new()
        } else {
            self.port.clone()
        }
    }

    pub(crate) fn normalize_special_port(&mut self) {
        if self.has_authority
            && default_port_for_scheme(&self.scheme)
                .is_some_and(|default_port| self.port == default_port)
        {
            self.port.clear();
        }
    }

    fn normalize_file_host_state(&mut self) {
        if !self.scheme.eq_ignore_ascii_case("file") {
            return;
        }
        if self.hostname.eq_ignore_ascii_case("localhost") {
            self.hostname.clear();
        }
        self.port.clear();
    }

    fn normalize_hostname_case(&mut self) {
        if self.has_authority {
            self.hostname = self.hostname.to_ascii_lowercase();
        }
    }

    fn has_network_file_host(&self) -> bool {
        self.has_authority
            && !self.hostname.is_empty()
            && !self.hostname.eq_ignore_ascii_case("localhost")
    }

    pub(crate) fn apply_protocol_setter(&mut self, next_scheme: &str) -> bool {
        let next_scheme = next_scheme.to_ascii_lowercase();
        let current_is_special = is_special_url_scheme(&self.scheme);
        let next_is_special = is_special_url_scheme(&next_scheme);
        let current_is_file = self.scheme.eq_ignore_ascii_case("file");
        let next_is_file = next_scheme == "file";

        if current_is_file {
            if next_is_file {
                self.scheme = next_scheme;
                self.normalize_file_host_state();
                return true;
            }
            if !next_is_special
                || !self.has_network_file_host()
                || !self.username.is_empty()
                || !self.password.is_empty()
                || !self.effective_port().is_empty()
            {
                return false;
            }
            self.scheme = next_scheme;
            self.normalize_special_port();
            return true;
        }

        if next_is_file
            && (!current_is_special
                || !self.has_authority
                || !self.username.is_empty()
                || !self.password.is_empty()
                || !self.effective_port().is_empty())
        {
            return false;
        }
        if !next_is_file && current_is_special != next_is_special {
            return false;
        }

        self.scheme = next_scheme;
        if next_is_file {
            self.normalize_file_host_state();
        } else {
            self.normalize_special_port();
        }
        true
    }

    pub(crate) fn apply_host_setter(&mut self, raw: &str) -> bool {
        if !self.has_authority {
            return false;
        }
        let parsed = parse_hostname_and_port(raw);
        if !parsed.valid_host {
            return false;
        }
        if self.scheme.eq_ignore_ascii_case("file") {
            if !matches!(parsed.port, ParsedAuthorityPort::Missing) {
                return false;
            }
            self.hostname = parsed.hostname;
            self.normalize_hostname_case();
            self.normalize_file_host_state();
            return true;
        }
        if parsed.hostname.is_empty() {
            return false;
        }
        self.hostname = parsed.hostname;
        if let ParsedAuthorityPort::Value(port) = parsed.port {
            self.port = port;
        }
        self.normalize_hostname_case();
        self.normalize_special_port();
        true
    }

    pub(crate) fn apply_hostname_setter(&mut self, raw: &str) -> bool {
        if !self.has_authority {
            return false;
        }
        let Some(hostname) = normalize_authority_hostname(raw) else {
            return false;
        };
        if !self.scheme.eq_ignore_ascii_case("file") && hostname.is_empty() {
            return false;
        }
        self.hostname = hostname;
        self.normalize_hostname_case();
        if self.scheme.eq_ignore_ascii_case("file") {
            self.normalize_file_host_state();
        } else {
            self.normalize_special_port();
        }
        true
    }

    pub(crate) fn apply_port_setter(&mut self, raw: &str) -> bool {
        if !self.has_authority || self.scheme.eq_ignore_ascii_case("file") {
            return false;
        }
        let Some(port) = normalized_url_port(raw) else {
            return false;
        };
        self.port = port;
        self.normalize_special_port();
        true
    }

    pub(crate) fn host(&self) -> String {
        let port = self.effective_port();
        if port.is_empty() {
            self.hostname.clone()
        } else {
            format!("{}:{}", self.hostname, port)
        }
    }

    pub(crate) fn origin(&self) -> String {
        if self.scheme.eq_ignore_ascii_case("file") {
            return "null".to_string();
        }
        if self.has_authority && !self.hostname.is_empty() {
            format!("{}//{}", self.protocol(), self.host())
        } else {
            "null".to_string()
        }
    }

    pub(crate) fn href(&self) -> String {
        if self.has_authority {
            let path = if self.pathname.is_empty() {
                "/".to_string()
            } else {
                self.pathname.clone()
            };
            let credentials = if self.username.is_empty() && self.password.is_empty() {
                String::new()
            } else if self.password.is_empty() {
                format!("{}@", self.username)
            } else {
                format!("{}:{}@", self.username, self.password)
            };
            format!(
                "{}//{}{}{}{}{}",
                self.protocol(),
                credentials,
                self.host(),
                path,
                self.search,
                self.hash
            )
        } else {
            format!(
                "{}{}{}{}",
                self.protocol(),
                self.opaque_path,
                self.search,
                self.hash
            )
        }
    }

    pub(crate) fn parse(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        let scheme_end = trimmed.find(':')?;
        let scheme = trimmed[..scheme_end].to_ascii_lowercase();
        if !is_valid_url_scheme(&scheme) {
            return None;
        }
        let raw_rest = &trimmed[scheme_end + 1..];
        let is_special_non_file =
            is_special_url_scheme(&scheme) && !scheme.eq_ignore_ascii_case("file");
        let normalized_special_rest = if is_special_non_file {
            let suffix_start = raw_rest
                .find(|ch| matches!(ch, '?' | '#'))
                .unwrap_or(raw_rest.len());
            let mut normalized = raw_rest[..suffix_start].replace('\\', "/");
            normalized.push_str(&raw_rest[suffix_start..]);
            Some(normalized)
        } else {
            None
        };
        let rest = normalized_special_rest.as_deref().unwrap_or(raw_rest);

        let authority_and_tail = if is_special_non_file {
            let without_leading_slashes = rest.trim_start_matches('/');
            if without_leading_slashes.is_empty()
                || without_leading_slashes.starts_with('?')
                || without_leading_slashes.starts_with('#')
            {
                return None;
            }
            Some(without_leading_slashes)
        } else {
            rest.strip_prefix("//")
        };

        if let Some(without_slashes) = authority_and_tail {
            let authority_end = without_slashes
                .find(|ch| ['/', '?', '#'].contains(&ch))
                .unwrap_or(without_slashes.len());
            let authority = &without_slashes[..authority_end];
            let tail = &without_slashes[authority_end..];
            let (username, password, hostname, port, port_state) =
                split_authority_components(authority)?;
            if is_special_non_file && hostname.is_empty() {
                return None;
            }
            let (pathname, search, hash) = split_path_search_hash(tail);
            let pathname = if pathname.is_empty() {
                "/".to_string()
            } else {
                normalize_pathname(&pathname)
            };
            let mut parts = Self {
                scheme,
                has_authority: true,
                username,
                password,
                hostname,
                port,
                pathname,
                opaque_path: String::new(),
                search,
                hash,
            };
            parts.normalize_hostname_case();
            if parts.scheme.eq_ignore_ascii_case("file")
                && (!parts.username.is_empty()
                    || !parts.password.is_empty()
                    || !matches!(port_state, ParsedAuthorityPort::Missing))
            {
                return None;
            }
            parts.normalize_file_host_state();
            parts.normalize_special_port();
            Some(parts)
        } else {
            let (opaque_path, search, hash) = split_opaque_search_hash(rest);
            let mut parts = Self {
                scheme,
                has_authority: false,
                username: String::new(),
                password: String::new(),
                hostname: String::new(),
                port: String::new(),
                pathname: String::new(),
                opaque_path,
                search,
                hash,
            };
            parts.normalize_special_port();
            Some(parts)
        }
    }
}

pub(crate) fn is_valid_url_scheme(scheme: &str) -> bool {
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
}

pub(crate) fn looks_like_absolute_url(input: &str) -> bool {
    let trimmed = input.trim();
    let Some(scheme_end) = trimmed.find(':') else {
        return false;
    };
    let scheme = &trimmed[..scheme_end];
    is_valid_url_scheme(scheme)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParsedAuthorityPort {
    Missing,
    Empty,
    Value(String),
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedHostnameAndPort {
    pub(crate) hostname: String,
    pub(crate) port: ParsedAuthorityPort,
    pub(crate) valid_host: bool,
}

pub(crate) fn is_special_url_scheme(scheme: &str) -> bool {
    matches!(
        scheme.to_ascii_lowercase().as_str(),
        "ftp" | "file" | "http" | "https" | "ws" | "wss"
    )
}

pub(crate) fn default_port_for_scheme(scheme: &str) -> Option<&'static str> {
    match scheme.to_ascii_lowercase().as_str() {
        "ftp" => Some("21"),
        "http" | "ws" => Some("80"),
        "https" | "wss" => Some("443"),
        _ => None,
    }
}

pub(crate) fn normalized_url_port(raw: &str) -> Option<String> {
    if raw.is_empty() {
        return Some(String::new());
    }
    if !raw.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let value = raw.parse::<u32>().ok()?;
    if value > u16::MAX as u32 {
        return None;
    }
    Some(value.to_string())
}

fn is_forbidden_host_code_point(ch: char) -> bool {
    ch.is_ascii_whitespace()
        || matches!(
            ch,
            '\0' | '#' | '%' | '/' | ':' | '<' | '>' | '?' | '@' | '[' | '\\' | ']' | '^' | '|'
        )
}

fn normalize_authority_hostname(hostname: &str) -> Option<String> {
    if hostname.is_empty() {
        return Some(String::new());
    }

    if let Some(body) = hostname.strip_prefix('[') {
        if !body.ends_with(']')
            || body.len() <= 1
            || body[..body.len() - 1].contains('[')
            || body[..body.len() - 1]
                .chars()
                .any(|ch| ch.is_ascii_whitespace() || ch == '%')
        {
            return None;
        }
        return Some(hostname.to_string());
    }

    if hostname.contains(['[', ']']) {
        return None;
    }

    let bytes = hostname.as_bytes();
    let mut decoded = String::with_capacity(hostname.len());
    let mut percent_bytes = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() {
                return None;
            }
            let hi = from_hex_digit(bytes[i + 1])?;
            let lo = from_hex_digit(bytes[i + 2])?;
            percent_bytes.push((hi << 4) | lo);
            i += 3;
            continue;
        }

        if !percent_bytes.is_empty() {
            decoded.push_str(std::str::from_utf8(&percent_bytes).ok()?);
            percent_bytes.clear();
        }
        let ch = hostname[i..].chars().next()?;
        decoded.push(ch);
        i += ch.len_utf8();
    }

    if !percent_bytes.is_empty() {
        decoded.push_str(std::str::from_utf8(&percent_bytes).ok()?);
    }

    if decoded.chars().any(is_forbidden_host_code_point) {
        return None;
    }

    let normalized = domain_to_ascii(&decoded).ok()?;
    if normalized.chars().any(is_forbidden_host_code_point) {
        return None;
    }
    Some(normalized)
}

pub(crate) fn parse_hostname_and_port(authority: &str) -> ParsedHostnameAndPort {
    if authority.is_empty() {
        return ParsedHostnameAndPort {
            hostname: String::new(),
            port: ParsedAuthorityPort::Missing,
            valid_host: true,
        };
    }

    if let Some(rest) = authority.strip_prefix('[') {
        if let Some(end_idx) = rest.find(']') {
            let raw_hostname = &authority[..end_idx + 2];
            let suffix = &authority[end_idx + 2..];
            let port = if suffix.is_empty() {
                ParsedAuthorityPort::Missing
            } else if let Some(raw_port) = suffix.strip_prefix(':') {
                if raw_port.is_empty() {
                    ParsedAuthorityPort::Empty
                } else if let Some(port) = normalized_url_port(raw_port) {
                    ParsedAuthorityPort::Value(port)
                } else {
                    ParsedAuthorityPort::Invalid
                }
            } else {
                return ParsedHostnameAndPort {
                    hostname: authority.to_string(),
                    port: ParsedAuthorityPort::Missing,
                    valid_host: false,
                };
            };
            let Some(hostname) = normalize_authority_hostname(raw_hostname) else {
                return ParsedHostnameAndPort {
                    hostname: raw_hostname.to_string(),
                    port,
                    valid_host: false,
                };
            };
            return ParsedHostnameAndPort {
                hostname,
                port,
                valid_host: true,
            };
        }
        return ParsedHostnameAndPort {
            hostname: authority.to_string(),
            port: ParsedAuthorityPort::Missing,
            valid_host: false,
        };
    }

    if authority.contains(['[', ']']) {
        return ParsedHostnameAndPort {
            hostname: authority.to_string(),
            port: ParsedAuthorityPort::Missing,
            valid_host: false,
        };
    }

    if let Some(idx) = authority.rfind(':') {
        let raw_hostname = &authority[..idx];
        let port = &authority[idx + 1..];
        if !raw_hostname.contains(':') {
            let port = if port.is_empty() {
                ParsedAuthorityPort::Empty
            } else if let Some(port) = normalized_url_port(port) {
                ParsedAuthorityPort::Value(port)
            } else {
                ParsedAuthorityPort::Invalid
            };
            let normalized_hostname = normalize_authority_hostname(raw_hostname);
            let hostname = normalized_hostname
                .clone()
                .unwrap_or_else(|| raw_hostname.to_string());
            return ParsedHostnameAndPort {
                hostname,
                port,
                valid_host: normalized_hostname.is_some(),
            };
        }
    }

    let normalized_hostname = normalize_authority_hostname(authority);
    let hostname = normalized_hostname
        .clone()
        .unwrap_or_else(|| authority.to_string());
    ParsedHostnameAndPort {
        hostname,
        port: ParsedAuthorityPort::Missing,
        valid_host: normalized_hostname.is_some(),
    }
}

pub(crate) fn split_authority_components(
    authority: &str,
) -> Option<(String, String, String, String, ParsedAuthorityPort)> {
    if authority.is_empty() {
        return Some((
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            ParsedAuthorityPort::Missing,
        ));
    }

    let (userinfo, hostport) = if let Some(at) = authority.rfind('@') {
        (&authority[..at], &authority[at + 1..])
    } else {
        ("", authority)
    };

    let (username, password) = if userinfo.is_empty() {
        (String::new(), String::new())
    } else if let Some((username, password)) = userinfo.split_once(':') {
        (username.to_string(), password.to_string())
    } else {
        (userinfo.to_string(), String::new())
    };

    let parsed = parse_hostname_and_port(hostport);
    if !parsed.valid_host {
        return None;
    }
    let port = match &parsed.port {
        ParsedAuthorityPort::Missing | ParsedAuthorityPort::Empty => String::new(),
        ParsedAuthorityPort::Value(port) => port.clone(),
        ParsedAuthorityPort::Invalid => return None,
    };
    Some((username, password, parsed.hostname, port, parsed.port))
}

pub(crate) fn split_path_search_hash(tail: &str) -> (String, String, String) {
    let mut pathname = tail;
    let mut search = "";
    let mut hash = "";

    if let Some(hash_pos) = tail.find('#') {
        pathname = &tail[..hash_pos];
        hash = &tail[hash_pos..];
    }

    if let Some(search_pos) = pathname.find('?') {
        search = &pathname[search_pos..];
        pathname = &pathname[..search_pos];
    }

    (pathname.to_string(), search.to_string(), hash.to_string())
}

pub(crate) fn split_opaque_search_hash(rest: &str) -> (String, String, String) {
    let mut opaque_path = rest;
    let mut search = "";
    let mut hash = "";

    if let Some(hash_pos) = rest.find('#') {
        opaque_path = &rest[..hash_pos];
        hash = &rest[hash_pos..];
    }

    if let Some(search_pos) = opaque_path.find('?') {
        search = &opaque_path[search_pos..];
        opaque_path = &opaque_path[..search_pos];
    }

    (
        opaque_path.to_string(),
        search.to_string(),
        hash.to_string(),
    )
}

pub(crate) fn normalize_pathname(pathname: &str) -> String {
    let starts_with_slash = pathname.starts_with('/');
    let ends_with_slash = pathname.ends_with('/') && pathname.len() > 1;
    let mut parts = Vec::new();
    for segment in pathname.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            parts.pop();
            continue;
        }
        parts.push(segment);
    }
    let mut out = if starts_with_slash {
        format!("/{}", parts.join("/"))
    } else {
        parts.join("/")
    };
    if out.is_empty() {
        out.push('/');
    }
    if ends_with_slash && !out.ends_with('/') {
        out.push('/');
    }
    out
}

pub(crate) fn ensure_search_prefix(value: &str) -> String {
    if value.is_empty() {
        String::new()
    } else if value.starts_with('?') {
        value.to_string()
    } else {
        format!("?{value}")
    }
}

pub(crate) fn ensure_hash_prefix(value: &str) -> String {
    if value.is_empty() {
        String::new()
    } else if value.starts_with('#') {
        value.to_string()
    } else {
        format!("#{value}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HistoryEntry {
    pub(crate) key: String,
    pub(crate) url: String,
    pub(crate) state: Value,
}

#[derive(Debug)]
pub(crate) struct LocationHistoryState {
    pub(crate) history_object: Rc<RefCell<ObjectValue>>,
    pub(crate) navigation_object: Rc<RefCell<ObjectValue>>,
    pub(crate) history_entries: Vec<HistoryEntry>,
    pub(crate) history_index: usize,
    pub(crate) next_history_entry_key: usize,
    pub(crate) history_scroll_restoration: String,
    pub(crate) location_mock_pages: HashMap<String, String>,
    pub(crate) location_navigations: Vec<LocationNavigation>,
    pub(crate) location_reload_count: usize,
}

impl LocationHistoryState {
    pub(crate) fn new(initial_url: &str) -> Self {
        Self {
            history_object: Rc::new(RefCell::new(ObjectValue::default())),
            navigation_object: Rc::new(RefCell::new(ObjectValue::default())),
            history_entries: vec![HistoryEntry {
                key: "entry-0".to_string(),
                url: initial_url.to_string(),
                state: Value::Null,
            }],
            history_index: 0,
            next_history_entry_key: 1,
            history_scroll_restoration: "auto".to_string(),
            location_mock_pages: HashMap::new(),
            location_navigations: Vec::new(),
            location_reload_count: 0,
        }
    }
}
