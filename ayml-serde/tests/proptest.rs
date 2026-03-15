//! Property-based tests for ayml-serde roundtrip and robustness.

use ayml_serde::{Value, from_str, to_string};
use proptest::prelude::*;
use std::collections::HashMap;

// ── Generators ───────────────────────────────────────────────────

/// Map keys: valid AYML mapping keys that roundtrip through serde's
/// HashMap<String, _>.
fn arb_map_key() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-zA-Z_][a-zA-Z0-9_]{0,15}",
        // Keys that need quoting
        Just(String::new()),
        Just("null".into()),
        Just("true".into()),
        Just("false".into()),
        Just("42".into()),
        Just("with: colon".into()),
        Just("has #hash".into()),
        Just("back\\slash".into()),
    ]
}

/// Scalar strings covering bare, quoted, and special cases.
fn arb_scalar_string() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple alphanumeric (bare strings)
        "[a-zA-Z][a-zA-Z0-9 _-]{0,30}",
        // Strings containing special chars that force quoting
        "[a-zA-Z0-9 :{}\\[\\]#,]{1,20}",
        // Empty string
        Just(String::new()),
        // Reserved words
        Just("null".into()),
        Just("true".into()),
        Just("false".into()),
        Just("inf".into()),
        Just("+inf".into()),
        Just("-inf".into()),
        Just("nan".into()),
        // Numeric-looking
        Just("42".into()),
        Just("3.25".into()),
        Just("-7".into()),
        // Strings with control characters (serializer escapes these)
        Just("tab\there".into()),
        Just("cr\rhere".into()),
        Just("newline\nhere".into()),
    ]
}

/// Generate a random Value tree with bounded depth.
fn arb_value(depth: u32) -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null(())),
        any::<bool>().prop_map(Value::Bool),
        prop_oneof![any::<i64>(), Just(i64::MIN), Just(i64::MAX), Just(0_i64),]
            .prop_map(Value::Int),
        // Finite, non-subnormal floats that roundtrip cleanly via ryu
        (-1e15_f64..1e15_f64)
            .prop_filter("finite and normal", |f| f.is_finite())
            .prop_map(|f| Value::Float((f * 1e6).round() / 1e6)),
        prop_oneof![
            Just(Value::Float(f64::INFINITY)),
            Just(Value::Float(f64::NEG_INFINITY)),
            Just(Value::Float(f64::NAN)),
            Just(Value::Float(0.0)),
            Just(Value::Float(1.0)),
            Just(Value::Float(-1.0)),
            Just(Value::Float(1e20)),
            Just(Value::Float(-1e20)),
        ],
        arb_scalar_string().prop_map(Value::Str),
    ];

    if depth == 0 {
        leaf.boxed()
    } else {
        prop_oneof![
            4 => leaf,
            // Sequences of 0..5 elements
            1 => prop::collection::vec(arb_value(depth - 1), 0..5)
                .prop_map(Value::Seq),
            // Mappings of 0..5 entries (String keys for serde HashMap)
            1 => prop::collection::vec(
                    (arb_map_key(), arb_value(depth - 1)),
                    0..5
                )
                .prop_map(|entries| {
                    let map: HashMap<String, Value> = entries.into_iter().collect();
                    Value::Map(map)
                }),
        ]
        .boxed()
    }
}

// ── Property tests ───────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// Serialize a random Value tree, deserialize it back, and verify equality.
    #[test]
    fn serde_round_trip(value in arb_value(4)) {
        let serialized = to_string(&value).map_err(|e| {
            TestCaseError::fail(format!(
                "serialize failed: {e}\n--- value ---\n{value:?}"
            ))
        })?;

        let deserialized: Value = from_str(&serialized).map_err(|e| {
            TestCaseError::fail(format!(
                "deserialize failed: {e}\n--- serialized ---\n{serialized}--- value ---\n{value:?}"
            ))
        })?;

        prop_assert!(
            value == deserialized,
            "values differ\n--- original ---\n{:?}\n--- serialized ---\n{}\n--- deserialized ---\n{:?}",
            value,
            serialized,
            deserialized,
        );
    }

    /// Ensure from_str never panics on arbitrary input.
    #[test]
    fn deserializer_no_panic(input in "[\\s\\S]{0,200}") {
        let _ = from_str::<Value>(&input);
    }

    /// Ensure to_string never panics on arbitrary Value trees.
    #[test]
    fn serializer_no_panic(value in arb_value(3)) {
        let _ = to_string(&value);
    }
}
