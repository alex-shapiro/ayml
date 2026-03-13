use ayml_core::{ErrorKind, parse, parse_with_max_depth};

#[test]
fn duplicate_key() {
    let input = "a: 1\na: 2";
    let err = parse(input).unwrap_err();
    assert!(matches!(err.kind, ErrorKind::DuplicateKey(_)));
}

#[test]
fn null_mapping_key() {
    let input = "null: value";
    let err = parse(input).unwrap_err();
    assert!(matches!(err.kind, ErrorKind::NullKey));
}

#[test]
fn float_mapping_key() {
    let input = "3.14: value";
    let err = parse(input).unwrap_err();
    assert!(matches!(err.kind, ErrorKind::FloatKey));
}

#[test]
fn invalid_escape_in_double_quoted() {
    let input = r#"s: "\q""#;
    let err = parse(input).unwrap_err();
    assert!(matches!(err.kind, ErrorKind::InvalidEscape(_)));
}

#[test]
fn unclosed_double_quote() {
    let input = r#"s: "hello"#;
    let err = parse(input).unwrap_err();
    assert!(matches!(err.kind, ErrorKind::UnexpectedEof));
}

#[test]
fn unclosed_flow_sequence() {
    let input = "items: [a, b";
    let err = parse(input).unwrap_err();
    assert!(matches!(err.kind, ErrorKind::Expected(_)));
}

#[test]
fn unclosed_flow_mapping() {
    let input = "m: {a: 1";
    let err = parse(input).unwrap_err();
    assert!(matches!(err.kind, ErrorKind::Expected(_)));
}

#[test]
fn bom_rejected() {
    let input = "\u{FEFF}key: value";
    let err = parse(input).unwrap_err();
    assert!(matches!(err.kind, ErrorKind::ByteOrderMark));
}

#[test]
fn deeply_nested_flow_sequences_rejected() {
    // 200 nested opening brackets — well past the default limit of 128.
    let input: String = "[".repeat(200) + &"]".repeat(200);
    let err = parse(&input).unwrap_err();
    assert!(matches!(err.kind, ErrorKind::RecursionLimit));
}

#[test]
fn deeply_nested_flow_mappings_rejected() {
    // {a: {a: {a: ... }}}
    let mut input = String::new();
    for _ in 0..200 {
        input.push_str("{a: ");
    }
    input.push_str("1");
    for _ in 0..200 {
        input.push('}');
    }
    let err = parse(&input).unwrap_err();
    assert!(matches!(err.kind, ErrorKind::RecursionLimit));
}

#[test]
fn deeply_nested_block_mappings_rejected() {
    // a:\n  b:\n    c:\n ... (200 levels deep)
    let mut input = String::new();
    for i in 0..200 {
        for _ in 0..(i * 2) {
            input.push(' ');
        }
        input.push_str(&format!("k{i}:\n"));
    }
    for _ in 0..(200 * 2) {
        input.push(' ');
    }
    input.push('1');
    let err = parse(&input).unwrap_err();
    assert!(matches!(err.kind, ErrorKind::RecursionLimit));
}

#[test]
fn custom_max_depth_is_respected() {
    // 5 levels of nesting: [[[[[ 1 ]]]]]
    let input = "[".repeat(5) + "1" + &"]".repeat(5);
    // Limit of 4 should reject it.
    let err = parse_with_max_depth(&input, 4).unwrap_err();
    assert!(matches!(err.kind, ErrorKind::RecursionLimit));
    // Limit of 10 should accept it.
    let node = parse_with_max_depth(&input, 10).unwrap();
    assert!(node.value.as_sequence().is_some());
}

#[test]
fn moderate_nesting_within_default_limit_ok() {
    // 50 levels — well within the default 128.
    let input = "[".repeat(50) + "1" + &"]".repeat(50);
    let node = parse(&input).unwrap();
    assert!(node.value.as_sequence().is_some());
}

#[test]
fn error_has_line_column() {
    let input = "a: 1\nb: 2\na: 3";
    let err = parse(input).unwrap_err();
    // The duplicate key error should have a meaningful line/column
    assert!(err.line >= 1);
    assert!(err.column >= 1);
}
