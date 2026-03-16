//! Error handling tests: invalid input, type mismatches, edge cases.

use ayml::from_str;

// ── Parse errors ────────────────────────────────────────────────

#[test]
fn error_empty_input() {
    assert!(from_str::<i32>("").is_err());
    assert!(from_str::<String>("").is_err());
    assert!(from_str::<bool>("").is_err());
}

#[test]
fn error_trailing_characters() {
    assert!(from_str::<i32>("42 extra").is_err());
    assert!(from_str::<bool>("true extra").is_err());
}

#[test]
fn error_type_mismatch_bool() {
    assert!(from_str::<bool>("42").is_err());
    assert!(from_str::<bool>("hello").is_err());
    assert!(from_str::<bool>("null").is_err());
}

#[test]
fn error_type_mismatch_int() {
    assert!(from_str::<i32>("hello").is_err());
    assert!(from_str::<i32>("true").is_err());
    assert!(from_str::<i32>("3.14").is_err());
}

#[test]
fn error_type_mismatch_float() {
    assert!(from_str::<f64>("hello").is_err());
    assert!(from_str::<f64>("true").is_err());
}

#[test]
fn error_integer_overflow_i8() {
    assert!(from_str::<i8>("128").is_err());
    assert!(from_str::<i8>("-129").is_err());
}

#[test]
fn error_integer_overflow_u8() {
    assert!(from_str::<u8>("256").is_err());
    assert!(from_str::<u8>("-1").is_err());
}

#[test]
fn error_integer_overflow_i64() {
    // Larger than i64::MAX
    assert!(from_str::<i64>("9999999999999999999").is_err());
}

#[test]
fn error_unclosed_string() {
    assert!(from_str::<String>(r#""hello"#).is_err());
}

#[test]
fn error_newline_in_string() {
    assert!(from_str::<String>("\"hello\nworld\"").is_err());
}

#[test]
fn error_invalid_escape() {
    assert!(from_str::<String>(r#""\q""#).is_err());
}

#[test]
fn error_invalid_hex_escape() {
    assert!(from_str::<String>(r#""\xGG""#).is_err());
}

#[test]
fn error_invalid_unicode_escape() {
    assert!(from_str::<String>(r#""\uZZZZ""#).is_err());
}

#[test]
fn error_unclosed_flow_sequence() {
    assert!(from_str::<Vec<i32>>("[1, 2").is_err());
}

#[test]
fn error_unclosed_flow_mapping() {
    assert!(from_str::<std::collections::HashMap<String, i32>>("{a: 1").is_err());
}

#[test]
fn error_missing_colon_in_mapping() {
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct S {
        #[allow(dead_code)]
        a: i32,
    }
    assert!(from_str::<S>("a 1").is_err());
}

#[test]
fn error_expected_sequence() {
    assert!(from_str::<Vec<i32>>("42").is_err());
}

#[test]
fn error_null_expected() {
    assert!(from_str::<()>("42").is_err());
    assert!(from_str::<()>("true").is_err());
    assert!(from_str::<()>("hello").is_err());
}

#[test]
fn error_char_too_long() {
    assert!(from_str::<char>("ab").is_err());
}

#[test]
fn error_char_empty() {
    assert!(from_str::<char>(r#""""#).is_err());
}

// ── Invalid UTF-8 ───────────────────────────────────────────────

#[test]
fn error_invalid_utf8_slice() {
    assert!(ayml::from_slice::<String>(&[0xFF, 0xFE]).is_err());
}

// ── Missing struct fields ───────────────────────────────────────

#[test]
fn error_missing_required_field() {
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct Config {
        #[allow(dead_code)]
        name: String,
        #[allow(dead_code)]
        port: u16,
    }
    // Only provides 'name', missing 'port'
    assert!(from_str::<Config>("name: app").is_err());
}

// ── Enum errors ─────────────────────────────────────────────────

#[test]
fn error_unknown_enum_variant() {
    use serde::Deserialize;
    #[derive(Deserialize)]
    enum Color {
        #[allow(dead_code)]
        Red,
        #[allow(dead_code)]
        Blue,
    }
    assert!(from_str::<Color>("Purple").is_err());
}

// ── Tabs for indentation ────────────────────────────────────────

#[test]
fn error_tab_indentation() {
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct S {
        #[allow(dead_code)]
        a: i32,
    }
    assert!(from_str::<S>("a: 1\n\tb: 2").is_err());
}

// ── from_reader errors ──────────────────────────────────────────

#[test]
fn error_from_reader_invalid_utf8() {
    let data: &[u8] = &[0xFF, 0xFE];
    assert!(ayml::from_reader::<_, String>(data).is_err());
}
