use ayml_core::{MapKey, Value, parse};

#[test]
fn top_level_comment() {
    let input = "# Server configuration\nhost: localhost\nport: 8080";
    let node = parse(input).unwrap();
    assert_eq!(node.comment.as_deref(), Some("Server configuration"));
}

#[test]
fn multiline_top_comment() {
    let input = "# Network connection rules\n# Use these rules to allow socket connections\nnetwork:\n  rules: ok";
    let node = parse(input).unwrap();
    assert_eq!(
        node.comment.as_deref(),
        Some("Network connection rules\nUse these rules to allow socket connections")
    );
}

#[test]
fn inline_comment_on_scalar() {
    let input = "hr:  65    # Home runs";
    let node = parse(input).unwrap();
    let map = node.value.as_mapping().unwrap();
    let hr_node = &map[&MapKey::String("hr".into())];
    assert_eq!(hr_node.value, Value::Int(65));
    assert_eq!(hr_node.inline_comment.as_deref(), Some("Home runs"));
}

#[test]
fn inline_comment_on_sequence_entry() {
    let input = "- 22 # Git (SSH)\n- 443 # Site";
    let node = parse(input).unwrap();
    let seq = node.value.as_sequence().unwrap();
    assert_eq!(seq[0].value, Value::Int(22));
    assert_eq!(seq[0].inline_comment.as_deref(), Some("Git (SSH)"));
    assert_eq!(seq[1].value, Value::Int(443));
    assert_eq!(seq[1].inline_comment.as_deref(), Some("Site"));
}

#[test]
fn hash_inside_triple_quoted_not_comment() {
    let input = "code: \"\"\"\n  # thread-safe work\n  mutex.lock()\n  \"\"\"";
    let node = parse(input).unwrap();
    let map = node.value.as_mapping().unwrap();
    let code = map[&MapKey::String("code".into())].value.as_str().unwrap();
    assert!(code.contains("# thread-safe work"));
}

#[test]
fn hash_preceded_by_nonspace_not_comment() {
    // In a bare string, `#` preceded by non-space is part of the string
    let input = "tag: foo#bar";
    let node = parse(input).unwrap();
    let map = node.value.as_mapping().unwrap();
    assert_eq!(
        map[&MapKey::String("tag".into())].value,
        Value::String("foo#bar".into())
    );
}
