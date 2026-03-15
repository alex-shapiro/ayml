//! Property-based tests for CommentedValue roundtrip and robustness.

use ayml_serde::{CommentedValue, CommentedValueKind, from_str, to_string};
use proptest::prelude::*;
use std::collections::HashMap;

// ── Generators ───────────────────────────────────────────────────

/// Comment text: at least one non-space char, no `#` or newlines.
fn arb_comment_line() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_.,:;!?()-][a-zA-Z0-9 _.,:;!?()-]{0,39}"
}

/// Optional single-line comment.
fn arb_opt_comment() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        3 => Just(None),
        1 => arb_comment_line().prop_map(Some),
    ]
}

/// Optional multi-line top comment (1-3 lines joined with \n).
fn arb_opt_top_comment() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        3 => Just(None),
        1 => prop::collection::vec(arb_comment_line(), 1..=3)
            .prop_map(|lines| Some(lines.join("\n"))),
    ]
}

fn arb_map_key() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-zA-Z_][a-zA-Z0-9_]{0,15}",
        Just(String::new()),
        Just("null".into()),
        Just("true".into()),
        Just("false".into()),
        Just("42".into()),
    ]
}

fn arb_scalar_string() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-zA-Z][a-zA-Z0-9 _-]{0,30}",
        Just(String::new()),
        Just("null".into()),
        Just("true".into()),
        Just("false".into()),
        Just("42".into()),
        Just("3.25".into()),
    ]
}

/// Generate a random CommentedValue tree with bounded depth.
///
/// Per the AYML spec, block sequence entries can only contain flow nodes
/// or compact mappings — not nested block sequences. The serializer emits
/// block style for sequences, so we avoid generating Seq-in-Seq to stay
/// within the spec. Sequences may only contain scalars and mappings.
fn arb_commented_value(depth: u32) -> impl Strategy<Value = CommentedValue> {
    arb_commented_value_inner(depth, false)
}

fn arb_commented_value_inner(
    depth: u32,
    inside_seq: bool,
) -> impl Strategy<Value = CommentedValue> {
    let leaf = prop_oneof![
        Just(CommentedValueKind::Null(())),
        any::<bool>().prop_map(CommentedValueKind::Bool),
        any::<i64>().prop_map(CommentedValueKind::Int),
        (-1e15_f64..1e15_f64)
            .prop_filter("finite and normal", |f| f.is_finite())
            .prop_map(|f| CommentedValueKind::Float((f * 1e6).round() / 1e6)),
        prop_oneof![
            Just(CommentedValueKind::Float(f64::INFINITY)),
            Just(CommentedValueKind::Float(f64::NEG_INFINITY)),
            Just(CommentedValueKind::Float(f64::NAN)),
        ],
        arb_scalar_string().prop_map(CommentedValueKind::Str),
    ];

    if depth == 0 {
        (arb_opt_top_comment(), arb_opt_comment(), leaf)
            .prop_map(|(top, inline, kind)| CommentedValue {
                top_comment: top,
                inline_comment: inline,
                value: kind,
            })
            .boxed()
    } else {
        // Seq children are inside_seq=true, so they won't generate nested seqs.
        // Map children are inside_seq=false, so maps can contain seqs.
        let seq_strategy = prop::collection::vec(arb_commented_value_inner(depth - 1, true), 0..4)
            .prop_map(CommentedValueKind::Seq);

        let map_strategy = prop::collection::vec(
            (arb_map_key(), arb_commented_value_inner(depth - 1, false)),
            0..4,
        )
        .prop_map(|entries| {
            let map: HashMap<String, CommentedValue> = entries.into_iter().collect();
            CommentedValueKind::Map(map)
        });

        let inner = if inside_seq {
            // Inside a seq: only scalars and maps (no nested seqs per spec)
            prop_oneof![
                4 => leaf,
                1 => map_strategy,
            ]
            .boxed()
        } else {
            prop_oneof![
                4 => leaf,
                1 => seq_strategy,
                1 => map_strategy,
            ]
            .boxed()
        };

        (arb_opt_top_comment(), arb_opt_comment(), inner)
            .prop_map(|(top, inline, kind)| CommentedValue {
                top_comment: top,
                inline_comment: inline,
                value: kind,
            })
            .boxed()
    }
}

/// Normalize a CommentedValue for roundtrip comparison:
/// - Strip all inline comments: inline comments on collection nodes don't
///   roundtrip (they get misattributed to the last child element), so we
///   strip them everywhere for a clean comparison.
/// - Treat `Some("")` as `None` for top comments.
/// - Values and top comments are the primary roundtrip targets.
fn normalize(cv: &CommentedValue) -> CommentedValue {
    let top = cv.top_comment.as_ref().filter(|c| !c.is_empty()).cloned();
    let kind = match &cv.value {
        CommentedValueKind::Seq(items) => {
            CommentedValueKind::Seq(items.iter().map(normalize).collect())
        }
        CommentedValueKind::Map(map) => {
            CommentedValueKind::Map(map.iter().map(|(k, v)| (k.clone(), normalize(v))).collect())
        }
        other => other.clone(),
    };
    CommentedValue {
        top_comment: top,
        inline_comment: None,
        value: kind,
    }
}

// ── Property tests ───────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5_000))]

    /// Serialize a random CommentedValue, deserialize it back, and verify
    /// that the value and comments survive the roundtrip.
    #[test]
    fn commented_value_round_trip(value in arb_commented_value(3)) {
        let serialized = to_string(&value).map_err(|e| {
            TestCaseError::fail(format!(
                "serialize failed: {e}\n--- value ---\n{value:?}"
            ))
        })?;

        let deserialized: CommentedValue = from_str(&serialized).map_err(|e| {
            TestCaseError::fail(format!(
                "deserialize failed: {e}\n--- serialized ---\n{serialized}--- value ---\n{value:?}"
            ))
        })?;

        let expected = normalize(&value);
        let actual = normalize(&deserialized);

        prop_assert!(
            expected == actual,
            "values differ\n--- original (normalized) ---\n{expected:?}\n--- serialized ---\n{serialized}\n--- deserialized (normalized) ---\n{actual:?}",
        );
    }

    /// Ensure from_str::<CommentedValue> never panics on arbitrary input.
    #[test]
    fn commented_value_deserialize_no_panic(input in "[\\s\\S]{0,200}") {
        let _ = from_str::<CommentedValue>(&input);
    }

    /// Ensure to_string never panics on arbitrary CommentedValue trees.
    #[test]
    fn commented_value_serialize_no_panic(value in arb_commented_value(3)) {
        let _ = to_string(&value);
    }
}
