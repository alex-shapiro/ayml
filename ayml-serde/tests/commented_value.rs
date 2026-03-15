use ayml_serde::{CommentedValue, CommentedValueKind};

#[test]
fn de_commented_value_scalar_with_inline() {
    let input = "42 # the answer\n";
    let cv: CommentedValue = ayml_serde::from_str(input).unwrap();
    assert_eq!(cv.inline_comment.as_deref(), Some("the answer"));
    assert!(matches!(cv.value, CommentedValueKind::Int(42)));
}

#[test]
fn de_commented_value_scalar_no_comment() {
    let input = "hello\n";
    let cv: CommentedValue = ayml_serde::from_str(input).unwrap();
    assert_eq!(cv.top_comment, None);
    assert_eq!(cv.inline_comment, None);
    assert!(matches!(cv.value, CommentedValueKind::Str(ref s) if s == "hello"));
}

#[test]
fn de_commented_value_map_with_comments() {
    let input = "\
# server config
host: localhost
# port number
port: 8080 # default
";
    let cv: CommentedValue = ayml_serde::from_str(input).unwrap();
    assert_eq!(cv.top_comment.as_deref(), Some("server config"));

    let map = match &cv.value {
        CommentedValueKind::Map(m) => m,
        other => panic!("expected map, got {other:?}"),
    };

    let host = &map["host"];
    assert!(matches!(host.value, CommentedValueKind::Str(ref s) if s == "localhost"));

    let port = &map["port"];
    assert!(matches!(port.value, CommentedValueKind::Int(8080)));
    assert_eq!(port.top_comment.as_deref(), Some("port number"));
    assert_eq!(port.inline_comment.as_deref(), Some("default"));
}

#[test]
fn roundtrip_commented_value_scalar() {
    let cv = CommentedValue {
        top_comment: None,
        inline_comment: Some("note".into()),
        value: CommentedValueKind::Str("test".into()),
    };
    let s = ayml_serde::to_string(&cv).unwrap();
    assert_eq!(s, "test # note\n");
    let cv2: CommentedValue = ayml_serde::from_str(&s).unwrap();
    assert_eq!(cv, cv2);
}

#[test]
fn roundtrip_commented_value_map() {
    let input = "\
host: localhost
port: 8080 # default
";
    let cv: CommentedValue = ayml_serde::from_str(input).unwrap();
    let output = ayml_serde::to_string(&cv).unwrap();
    // Re-parse and compare values (map order may differ)
    let cv2: CommentedValue = ayml_serde::from_str(&output).unwrap();
    assert_eq!(cv, cv2);
}

#[test]
fn de_commented_value_seq() {
    let input = "\
- 1
- 2 # second
- 3
";
    let cv: CommentedValue = ayml_serde::from_str(input).unwrap();
    let seq = match &cv.value {
        CommentedValueKind::Seq(s) => s,
        other => panic!("expected seq, got {other:?}"),
    };
    assert_eq!(seq.len(), 3);
    assert!(matches!(seq[0].value, CommentedValueKind::Int(1)));
    assert!(matches!(seq[1].value, CommentedValueKind::Int(2)));
    assert_eq!(seq[1].inline_comment.as_deref(), Some("second"));
    assert!(matches!(seq[2].value, CommentedValueKind::Int(3)));
}

#[test]
fn de_commented_value_null() {
    let input = "null\n";
    let cv: CommentedValue = ayml_serde::from_str(input).unwrap();
    assert!(matches!(cv.value, CommentedValueKind::Null(())));
}

#[test]
fn de_commented_value_bool() {
    let input = "true # yes\n";
    let cv: CommentedValue = ayml_serde::from_str(input).unwrap();
    assert!(matches!(cv.value, CommentedValueKind::Bool(true)));
    assert_eq!(cv.inline_comment.as_deref(), Some("yes"));
}

#[test]
fn de_commented_value_float() {
    let input = "3.25\n";
    let cv: CommentedValue = ayml_serde::from_str(input).unwrap();
    assert!(matches!(cv.value, CommentedValueKind::Float(v) if v == 3.25));
}

#[test]
fn display_commented_value() {
    let cv = CommentedValue {
        top_comment: Some("ignored by display".into()),
        inline_comment: None,
        value: CommentedValueKind::Int(42),
    };
    assert_eq!(format!("{cv}"), "42");
}
