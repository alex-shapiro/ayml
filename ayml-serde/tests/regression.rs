//! Regression tests for bugs found during code review.
//!
//! Each test is marked with `#[should_panic]` or asserts the buggy behavior
//! BEFORE the fix is applied. After fixing, the attribute/assertion is
//! updated to verify the correct behavior.

use ayml_serde::{from_str, to_string};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[test]
fn triple_quoted_nul_at_end_of_line() {
    // A triple-quoted string with \0 at end of a line, followed by more content
    let input = "\"\"\"\n  hello\\0\n  world\n  \"\"\"";
    let result: String = from_str(input).unwrap();
    // After fix: \0 should be preserved and newline should remain
    assert_eq!(result, "hello\0\nworld");
}

#[test]
fn ser_tab_before_hash_needs_quoting() {
    let input = "hello\t#world";
    let serialized = to_string(&input).unwrap();
    // After fix: must be quoted because \t# would start a comment
    assert!(
        serialized.starts_with('"'),
        "expected quoted output for tab-before-#, got: {serialized}"
    );
    let back: String = from_str(&serialized).unwrap();
    assert_eq!(back, input);
}

#[test]
fn ser_c1_control_char_needs_quoting() {
    let input = "hello\u{0080}world"; // U+0080 is a C1 control char
    let serialized = to_string(&input).unwrap();
    assert!(
        serialized.starts_with('"'),
        "expected quoted output for C1 control char, got: {serialized}"
    );
    let back: String = from_str(&serialized).unwrap();
    assert_eq!(back, input);
}

#[test]
fn ser_c1_control_char_u009f() {
    let input = "test\u{009F}value";
    let serialized = to_string(&input).unwrap();
    assert!(
        serialized.starts_with('"'),
        "expected quoted output for U+009F, got: {serialized}"
    );
    let back: String = from_str(&serialized).unwrap();
    assert_eq!(back, input);
}

#[test]
fn ser_c1_nel_u0085_allowed_bare() {
    // U+0085 (NEL) IS in c-printable and is nb-char (not a line break per spec).
    // It should be allowed in bare strings without quoting.
    let input = "test\u{0085}value";
    let serialized = to_string(&input).unwrap();
    let back: String = from_str(&serialized).unwrap();
    assert_eq!(back, input, "NEL should round-trip correctly");
}

#[test]
fn ser_triple_quote_in_content() {
    let input = "line1\nfoo\"\"\"bar";
    let serialized = to_string(&input).unwrap();
    let back: String = from_str(&serialized).unwrap();
    assert_eq!(
        back, input,
        "roundtrip failed for string containing \"\"\"\nayml:\n{serialized}"
    );
}

#[test]
fn ser_triple_quote_alone_on_line() {
    let input = "before\n\"\"\"\nafter";
    let serialized = to_string(&input).unwrap();
    let back: String = from_str(&serialized).unwrap();
    assert_eq!(
        back, input,
        "roundtrip failed for string with \"\"\" on its own line\nayml:\n{serialized}"
    );
}

#[test]
fn ser_struct_field_renamed_to_reserved() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct S {
        #[serde(rename = "null")]
        field: i32,
    }
    let val = S { field: 42 };
    let serialized = to_string(&val).unwrap();
    let back: S = from_str(&serialized).unwrap();
    assert_eq!(
        back, val,
        "roundtrip failed for struct with 'null' field\nayml:\n{serialized}"
    );
}

#[test]
fn ser_struct_field_with_colon_space() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct S {
        #[serde(rename = "key: value")]
        field: i32,
    }
    let val = S { field: 42 };
    let serialized = to_string(&val).unwrap();
    let back: S = from_str(&serialized).unwrap();
    assert_eq!(
        back, val,
        "roundtrip failed for struct with 'key: value' field\nayml:\n{serialized}"
    );
}

#[test]
fn de_option_quoted_null_key_as_map() {
    let input = "\"null\": 5";
    let result: Option<HashMap<String, i32>> = from_str(input).unwrap();
    assert!(
        result.is_some(),
        "expected Some for '\"null\": 5' as Option<Map>"
    );
    let map = result.unwrap();
    assert_eq!(map.get("null"), Some(&5));
}

#[test]
fn de_option_plain_null_is_none() {
    let result: Option<i32> = from_str("null").unwrap();
    assert_eq!(result, None);
}

#[test]
fn ser_unit_struct_as_key_rejected() {
    #[derive(Serialize, PartialEq, Eq, Hash)]
    struct Marker;

    let mut map = HashMap::new();
    map.insert(Marker, "value");
    let result = to_string(&map);
    assert!(
        result.is_err(),
        "expected error when using unit struct as map key"
    );
}

#[test]
fn ser_flow_indicator_in_string_roundtrip() {
    let cases = vec!["a,b", "a]b", "a}b", "[hello", "{hello"];
    for input in cases {
        let serialized = to_string(&input).unwrap();
        let back: String = from_str(&serialized).unwrap();
        assert_eq!(
            back, input,
            "roundtrip failed for string with flow indicator: {input}\nayml:\n{serialized}"
        );
    }
}

#[test]
fn ser_flow_indicators_in_map_values() {
    use std::collections::BTreeMap;
    let mut map = BTreeMap::new();
    map.insert("key".to_string(), "a,b".to_string());
    let serialized = to_string(&map).unwrap();
    let back: BTreeMap<String, String> = from_str(&serialized).unwrap();
    assert_eq!(
        back, map,
        "roundtrip failed for map with flow indicator in value\nayml:\n{serialized}"
    );
}

#[test]
fn ser_flow_indicator_in_seq_element() {
    let items = vec!["a,b".to_string(), "c]d".to_string()];
    let serialized = to_string(&items).unwrap();
    let back: Vec<String> = from_str(&serialized).unwrap();
    assert_eq!(
        back, items,
        "roundtrip failed for seq with flow indicators\nayml:\n{serialized}"
    );
}

#[test]
fn value_display_str_null_ambiguity() {
    use ayml_serde::Value;
    let v = Value::Str("null".to_string());
    let display = format!("{v}");
    assert_ne!(
        display, "null",
        "Str(\"null\") should not display as bare 'null'"
    );
}

#[test]
fn value_display_float_int_ambiguity() {
    use ayml_serde::Value;
    let v = Value::Float(1.0);
    let display = format!("{v}");
    assert!(
        display.contains('.'),
        "Float(1.0) should display with decimal point, got: {display}"
    );
}

#[test]
fn commented_value_display_str_null_ambiguity() {
    use ayml_serde::{Commented, CommentedValueKind};
    let v = Commented {
        top_comment: None,
        value: CommentedValueKind::Str("null".to_string()),
        inline_comment: None,
    };
    let display = format!("{v}");
    assert_ne!(
        display, "null",
        "Str(\"null\") should not display as bare 'null'"
    );
}

#[test]
fn commented_value_display_float_int_ambiguity() {
    use ayml_serde::{Commented, CommentedValueKind};
    let v = Commented {
        top_comment: None,
        value: CommentedValueKind::Float(1.0),
        inline_comment: None,
    };
    let display = format!("{v}");
    assert!(
        display.contains('.'),
        "Float(1.0) should display with decimal point, got: {display}"
    );
}

#[test]
fn de_reject_unquoted_null_key() {
    let input = "null: 5";
    // When target is Value (untyped), null key should be rejected
    let result: Result<ayml_serde::Value, _> = from_str(input);
    assert!(
        result.is_err(),
        "expected error for unquoted null mapping key"
    );
}

#[test]
fn de_reject_unquoted_nan_key() {
    let input = "nan: 5";
    let result: Result<ayml_serde::Value, _> = from_str(input);
    assert!(
        result.is_err(),
        "expected error for unquoted nan mapping key"
    );
}

#[test]
fn de_reject_unquoted_inf_key() {
    let input = "inf: 5";
    let result: Result<ayml_serde::Value, _> = from_str(input);
    assert!(
        result.is_err(),
        "expected error for unquoted inf mapping key"
    );
}

#[test]
fn de_reject_unquoted_float_key() {
    let input = "3.14: 5";
    let result: Result<ayml_serde::Value, _> = from_str(input);
    assert!(
        result.is_err(),
        "expected error for unquoted float mapping key"
    );
}

#[test]
fn de_allow_quoted_null_key() {
    let input = "\"null\": 5";
    let result: Result<HashMap<String, i32>, _> = from_str(input);
    assert!(result.is_ok(), "quoted null key should be allowed");
}

#[test]
fn de_allow_quoted_nan_key() {
    let input = "\"nan\": 5";
    let result: Result<HashMap<String, i32>, _> = from_str(input);
    assert!(result.is_ok(), "quoted nan key should be allowed");
}

#[test]
fn ser_trailing_newline_roundtrip() {
    let input = "hello\n";
    let serialized = to_string(&input).unwrap();
    let back: String = from_str(&serialized).unwrap();
    assert_eq!(
        back, input,
        "trailing newline roundtrip failed\nayml:\n{serialized}"
    );
}

#[test]
fn ser_double_trailing_newline_roundtrip() {
    let input = "hello\n\n";
    let serialized = to_string(&input).unwrap();
    let back: String = from_str(&serialized).unwrap();
    assert_eq!(
        back, input,
        "double trailing newline roundtrip failed\nayml:\n{serialized}"
    );
}

#[test]
fn ser_only_newline_roundtrip() {
    let input = "\n";
    let serialized = to_string(&input).unwrap();
    let back: String = from_str(&serialized).unwrap();
    assert_eq!(
        back, input,
        "single newline roundtrip failed\nayml:\n{serialized}"
    );
}

#[test]
fn value_display_leading_space() {
    use ayml_serde::Value;
    let v = Value::Str(" hello".to_string());
    let display = format!("{v}");
    assert!(
        display.starts_with('"'),
        "Str(\" hello\") should be quoted, got: {display}"
    );
}

#[test]
fn value_display_trailing_space() {
    use ayml_serde::Value;
    let v = Value::Str("hello ".to_string());
    let display = format!("{v}");
    assert!(
        display.starts_with('"'),
        "Str(\"hello \") should be quoted, got: {display}"
    );
}

#[test]
fn value_display_dash_space_prefix() {
    use ayml_serde::Value;
    let v = Value::Str("- item".to_string());
    let display = format!("{v}");
    assert!(
        display.starts_with('"'),
        "Str(\"- item\") should be quoted, got: {display}"
    );
}

#[test]
fn looks_like_number_infinity_not_ayml_float() {
    // "infinity" is accepted by Rust's f64::parse but is NOT an AYML float.
    // The spec only recognizes "inf", "+inf", "-inf", "nan".
    // "infinity" should display as a bare string, not quoted.
    use ayml_serde::Value;
    let v = Value::Str("infinity".into());
    let display = format!("{v}");
    assert_eq!(
        display, "infinity",
        "\"infinity\" is not an AYML keyword and should not be quoted, got: {display}"
    );
}

#[test]
fn looks_like_number_nan_caps_not_ayml_float() {
    use ayml_serde::Value;
    let v = Value::Str("NaN".into());
    let display = format!("{v}");
    assert_eq!(
        display, "NaN",
        "\"NaN\" is not an AYML keyword (only \"nan\" is) and should not be quoted, got: {display}"
    );
}

#[test]
fn looks_like_number_dot_prefix_not_ayml_float() {
    // ".5" is accepted by Rust's f64::parse but AYML requires digits before the dot
    use ayml_serde::Value;
    let v = Value::Str(".5".into());
    let display = format!("{v}");
    assert_eq!(
        display, ".5",
        "\".5\" is not an AYML float and should not be quoted, got: {display}"
    );
}

#[test]
fn looks_like_number_trailing_dot_not_ayml_float() {
    // "5." is accepted by Rust's f64::parse but AYML requires digits after the dot
    use ayml_serde::Value;
    let v = Value::Str("5.".into());
    let display = format!("{v}");
    assert_eq!(
        display, "5.",
        "\"5.\" is not an AYML float and should not be quoted, got: {display}"
    );
}
