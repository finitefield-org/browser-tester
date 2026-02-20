pub(crate) fn is_ident(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
        return false;
    }

    chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

pub(crate) fn identifier_allows_regex_start(
    ident: &[u8],
    previous_significant: Option<u8>,
) -> bool {
    if matches!(previous_significant, Some(b'.')) {
        return false;
    }
    matches!(
        ident,
        b"return"
            | b"throw"
            | b"case"
            | b"delete"
            | b"typeof"
            | b"void"
            | b"yield"
            | b"await"
            | b"in"
            | b"of"
            | b"instanceof"
    )
}
