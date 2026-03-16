//! Shared formatting helpers for `Value` and `CommentedValueKind` display.

use std::fmt;

/// Display a string, quoting it if it could be ambiguous with other AYML types.
pub(crate) fn display_str(f: &mut fmt::Formatter<'_>, s: &str) -> fmt::Result {
    let needs_quoting = matches!(
        s,
        "null" | "true" | "false" | "inf" | "+inf" | "-inf" | "nan"
    ) || s.is_empty()
        || s.contains(['"', '\\', '\n', ':', ',', '[', ']', '{', '}', '#'])
        || looks_like_number(s);
    if needs_quoting {
        write!(f, "\"")?;
        for ch in s.chars() {
            match ch {
                '"' => write!(f, "\\\"")?,
                '\\' => write!(f, "\\\\")?,
                '\n' => write!(f, "\\n")?,
                '\r' => write!(f, "\\r")?,
                '\t' => write!(f, "\\t")?,
                c => write!(f, "{c}")?,
            }
        }
        write!(f, "\"")
    } else {
        write!(f, "{s}")
    }
}

/// Check if a string looks like it would parse as a number.
pub(crate) fn looks_like_number(s: &str) -> bool {
    let s = s
        .strip_prefix('+')
        .or_else(|| s.strip_prefix('-'))
        .unwrap_or(s);
    if s.is_empty() {
        return false;
    }
    if s.starts_with("0b") || s.starts_with("0o") || s.starts_with("0x") {
        return true;
    }
    if s.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    if (s.contains('.') || s.contains('e') || s.contains('E')) && s.as_bytes()[0].is_ascii_digit() {
        return true;
    }
    false
}
