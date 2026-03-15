//! Regression tests for bugs found during code review.
//!
//! Each test is marked with `#[should_panic]` or asserts the buggy behavior
//! BEFORE the fix is applied. After fixing, the attribute/assertion is
//! updated to verify the correct behavior.

use ayml_serde::{from_str, to_string};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── BUG 1: Triple-quoted `\0` sentinel collision ────────────────
//
// Line continuation uses '\x00' as internal sentinel. A `\0` escape at
// end-of-line followed by another content line has its NUL + newline stripped.

#[test]
fn triple_quoted_nul_at_end_of_line() {
    // A triple-quoted string with \0 at end of a line, followed by more content
    let input = "\"\"\"\n  hello\\0\n  world\n  \"\"\"";
    let result: String = from_str(input).unwrap();
    // After fix: \0 should be preserved and newline should remain
    assert_eq!(result, "hello\0\nworld");
}

// ── BUG 2: needs_quoting misses tab-before-# ────────────────────
//
// `needs_quoting` only checks for space before `#`, but tab before `#`
// also terminates a bare string.

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

// ── BUG 3: needs_quoting misses C1 control chars ────────────────
//
// Characters U+0080-U+009F (except U+0085) are non-printable per spec
// but multi-byte in UTF-8, so the byte-level check misses them.

#[test]
fn ser_c1_control_char_needs_quoting() {
    let input = "hello\u{0080}world"; // U+0080 is a C1 control char
    let serialized = to_string(&input).unwrap();
    // After fix: must be quoted and the C1 char escaped
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

// ── BUG 4: Triple-quoted strings containing """ ─────────────────
//
// The serializer doesn't escape `"` in triple-quoted content, so `"""`
// in the string prematurely closes the triple-quoted block.

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

// ── BUG 5: SerializeStruct field names not quoted ────────────────
//
// `SerializeStruct::serialize_field` writes keys via `write_str`
// instead of `write_key`, so renamed fields aren't quoted.

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

// ── Verify: deserialize_option correctly handles null followed by `:` ──
//
// `null: 5` with unquoted null key is rejected by key validation.
// But quoted "null" key works for Option<HashMap>.

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
    // Standalone `null` (no colon) should deserialize as None
    let result: Option<i32> = from_str("null").unwrap();
    assert_eq!(result, None);
}

// ── BUG 7: serialize_unit_struct bypasses serializing_key check ──
//
// `serialize_unit_struct` writes "null" without checking serializing_key.

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

// ── BUG 8: needs_quoting misses flow indicators mid-string ──────
//
// Characters like `,`, `]`, `}` mid-string would break in flow context.

#[test]
fn ser_flow_indicator_in_string_roundtrip() {
    // These strings contain flow indicators that could break parsing
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

// Flow indicators in map values (which get serialized in block context)
// should still roundtrip even if not quoted
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

// The real danger is flow indicators inside sequences (flow context)
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

// ── BUG 9: Value/CommentedValueKind Display ambiguity ────────────
//
// Str("null") displays as `null`, identical to Null.
// Float(1.0) displays as `1`, identical to Int(1).

#[test]
fn value_display_str_null_ambiguity() {
    use ayml_serde::Value;
    let v = Value::Str("null".to_string());
    let display = format!("{v}");
    // After fix: should be quoted to distinguish from null
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
    // After fix: should always include decimal point
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

// ── BUG 10: Unquoted null/float/nan/inf keys accepted ────────────
//
// Spec: "A mapping key MUST NOT resolve to null or float."

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
