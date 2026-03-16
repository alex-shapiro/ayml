use ayml_core::{MapKey, Node, Value};
use indexmap::IndexMap;
use proptest::prelude::*;

/// Generate a random MapKey.
fn arb_map_key() -> impl Strategy<Value = MapKey> {
    prop_oneof![
        any::<bool>().prop_map(MapKey::Bool),
        prop_oneof![any::<i64>(), Just(i64::MIN), Just(i64::MAX), Just(0_i64),]
            .prop_map(MapKey::Int),
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
        .prop_map(MapKey::String),
    ]
}

/// Generate a random string that is safe for AYML bare or quoted scalars.
fn arb_scalar_string() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple alphanumeric
        "[a-zA-Z][a-zA-Z0-9 _-]{0,30}",
        // Strings containing special chars that force quoting
        "[a-zA-Z0-9 :{}\\[\\]#,]{1,20}",
        // Empty string (needs quoting)
        Just(String::new()),
        // Reserved words (must round-trip as strings, not parsed values)
        Just("null".into()),
        Just("true".into()),
        Just("false".into()),
        Just("inf".into()),
        Just("+inf".into()),
        Just("-inf".into()),
        Just("nan".into()),
        // Strings that look like numbers
        Just("42".into()),
        Just("3.14".into()),
        Just("-7".into()),
        // Strings with control characters
        Just("tab\there".into()),
        Just("cr\rhere".into()),
        // Multiline strings (exercises triple-quoting)
        Just("line one\nline two".into()),
        Just("trailing newline\n".into()),
        Just("three\nlines\nhere".into()),
        // Trailing newline edge cases
        Just("end\n\n".into()),
        Just("\n".into()),
        Just("a\nb\n".into()),
        // Leading/trailing whitespace
        Just(" leading".into()),
        Just("trailing ".into()),
        Just(" both ".into()),
        // Dash-space prefix (looks like sequence indicator)
        Just("- item".into()),
        Just("- ".into()),
    ]
}

/// Generate a random Value tree with bounded depth.
fn arb_value(depth: u32) -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        prop_oneof![
            any::<i64>(),
            Just(i64::MIN),
            Just(i64::MAX),
            Just(i64::MIN + 1),
            Just(i64::MAX - 1),
            Just(0_i64),
        ]
        .prop_map(Value::Int),
        // Use finite, non-subnormal floats that round-trip cleanly
        (-1e15_f64..1e15_f64)
            .prop_filter("finite and normal", |f| f.is_finite())
            .prop_map(|f| Value::Float((f * 1e6).round() / 1e6)),
        prop_oneof![
            Just(Value::Float(f64::INFINITY)),
            Just(Value::Float(f64::NEG_INFINITY)),
            Just(Value::Float(f64::NAN)),
            Just(Value::Float(0.0)),
            Just(Value::Float(-0.0)),
            Just(Value::Float(1.0)),
            Just(Value::Float(-1.0)),
            Just(Value::Float(1e20)),
            Just(Value::Float(-1e20)),
            Just(Value::Float(1.5e-10)),
            Just(Value::Float(-1.5e-10)),
            Just(Value::Float(f64::MAX)),
            Just(Value::Float(f64::MIN)),
            Just(Value::Float(f64::MIN_POSITIVE)),
        ],
        arb_scalar_string().prop_map(Value::Str),
    ];

    if depth == 0 {
        leaf.boxed()
    } else {
        prop_oneof![
            4 => leaf,
            // Sequences of 0..5 elements
            1 => prop::collection::vec(arb_node(depth - 1), 0..5)
                .prop_map(Value::Seq),
            // Mappings of 0..5 entries
            1 => prop::collection::vec(
                    (arb_map_key(), arb_node(depth - 1)),
                    0..5
                )
                .prop_map(|entries| {
                    let map: IndexMap<MapKey, Node> = entries.into_iter().collect();
                    Value::Map(map)
                }),
        ]
        .boxed()
    }
}

/// Generate a random Node (no comments — comments don't affect value round-tripping).
fn arb_node(depth: u32) -> BoxedStrategy<Node> {
    arb_value(depth).prop_map(Node::new).boxed()
}

/// Compare two Values, treating NaN == NaN.
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => (a.is_nan() && b.is_nan()) || a == b,
        (Value::Str(a), Value::Str(b)) => a == b,
        (Value::Seq(a), Value::Seq(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(a, b)| values_equal(&a.value, &b.value))
        }
        (Value::Map(a), Value::Map(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, v)| b.get(k).is_some_and(|bv| values_equal(&v.value, &bv.value)))
        }
        _ => false,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// Emit a random Value tree, parse it back, and verify semantic equality.
    #[test]
    fn round_trip_random_values(node in arb_node(4)) {
        let emitted = ayml_core::emit(&node);

        let parsed = ayml_core::parse(&emitted).map_err(|e| {
            TestCaseError::fail(format!(
                "parse failed: {e}\n--- emitted ---\n{emitted}---"
            ))
        })?;

        prop_assert!(
            values_equal(&node.value, &parsed.value),
            "values differ\n--- original ---\n{:?}\n--- emitted ---\n{}\n--- parsed ---\n{:?}",
            node.value,
            emitted,
            parsed.value,
        );
    }

    /// Ensure the parser never panics on arbitrary input.
    #[test]
    fn parser_no_panic(input in "[\\s\\S]{0,200}") {
        // We don't care about the result — just that it doesn't panic.
        let _ = ayml_core::parse(&input);
    }
}
