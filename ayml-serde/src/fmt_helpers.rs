//! Shared formatting helpers for `Value` and `CommentedValueKind` display.

use std::fmt;

/// Display a string, quoting it if it could be ambiguous with other AYML types.
pub(crate) fn display_str(f: &mut fmt::Formatter<'_>, s: &str) -> fmt::Result {
    let needs_quoting = matches!(
        s,
        "null" | "true" | "false" | "inf" | "+inf" | "-inf" | "nan"
    ) || s.is_empty()
        || s.contains(['"', '\\', '\n', ':', ',', '[', ']', '{', '}', '#'])
        || s.starts_with(' ')
        || s.starts_with('\t')
        || s.starts_with('-')
        || s.ends_with(' ')
        || s.ends_with('\t')
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

    // Check decimal integer: optional sign followed by digits only
    if s.parse::<i64>().is_ok() {
        return true;
    }

    // Check AYML float grammar (not Rust's f64::parse, which is more permissive).
    // AYML requires: digits '.' digits (optional exponent), or digits exponent.
    // "inf"/"+inf"/"-inf"/"nan" are handled by the reserved-word check above.
    looks_like_ayml_float(unsigned)
}

/// Check if `s` matches AYML's float grammar (without sign prefix or special words).
/// Grammar: `digits '.' digits exponent?` or `digits exponent`.
fn looks_like_ayml_float(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;

    // Must start with at least one digit
    if i >= bytes.len() || !bytes[i].is_ascii_digit() {
        return false;
    }
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }

    if i < bytes.len() && bytes[i] == b'.' {
        // Fixed/exponential form: digits '.' digits exponent?
        i += 1;
        let dot_pos = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        // Must have at least one digit after '.'
        if i == dot_pos {
            return false;
        }
        // Optional exponent
        if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
            return looks_like_exponent(&bytes[i..]);
        }
        return i == bytes.len();
    }

    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        // Pure exponential form: digits exponent
        return looks_like_exponent(&bytes[i..]);
    }

    false
}

/// Check if bytes starting at 'e'/'E' form a valid exponent: `[eE] [+-]? digits+`
fn looks_like_exponent(bytes: &[u8]) -> bool {
    let mut i = 1; // skip 'e'/'E'
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    let start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    i > start && i == bytes.len()
}
