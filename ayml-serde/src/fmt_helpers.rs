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
///
/// Display an f64 in AYML format: nan, inf, -inf, or decimal with `.0` suffix.
pub(crate) fn display_float(f: &mut fmt::Formatter<'_>, v: f64) -> fmt::Result {
    if v.is_nan() {
        write!(f, "nan")
    } else if v.is_infinite() {
        if v.is_sign_positive() {
            write!(f, "inf")
        } else {
            write!(f, "-inf")
        }
    } else {
        let s = format!("{v}");
        if s.contains('.') || s.contains('e') || s.contains('E') {
            write!(f, "{s}")
        } else {
            write!(f, "{s}.0")
        }
    }
}

/// Matches the AYML parser's integer and float resolution: decimal integers,
/// `0b`/`0o`/`0x` prefixed integers, and standard floats.
pub(crate) fn looks_like_number(s: &str) -> bool {
    let unsigned = s
        .strip_prefix('+')
        .or_else(|| s.strip_prefix('-'))
        .unwrap_or(s);

    if let Some(bin) = unsigned.strip_prefix("0b") {
        return !bin.is_empty() && bin.chars().all(|c| c == '0' || c == '1');
    }
    if let Some(oct) = unsigned.strip_prefix("0o") {
        return !oct.is_empty() && oct.chars().all(|c| matches!(c, '0'..='7'));
    }
    if let Some(hex) = unsigned.strip_prefix("0x") {
        return !hex.is_empty() && hex.chars().all(|c| c.is_ascii_hexdigit());
    }

    s.parse::<i64>().is_ok() || s.parse::<f64>().is_ok()
}
