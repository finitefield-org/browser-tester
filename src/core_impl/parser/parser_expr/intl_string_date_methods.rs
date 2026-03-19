use super::*;

pub(crate) fn collect_top_level_char_positions(src: &str, target: u8) -> Vec<usize> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        if scanner.is_top_level() && bytes[i] == target {
            out.push(i);
        }
        i = scanner.advance(bytes, i);
    }

    out
}
