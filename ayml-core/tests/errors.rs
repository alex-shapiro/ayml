use ayml_core::{ErrorKind, parse};

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
fn error_has_line_column() {
    let input = "a: 1\nb: 2\na: 3";
    let err = parse(input).unwrap_err();
    // The duplicate key error should have a meaningful line/column
    assert!(err.line >= 1);
    assert!(err.column >= 1);
}
