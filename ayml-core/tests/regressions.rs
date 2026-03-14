use ayml_core::{MapKey, Node, Value, emit, parse};
use indexmap::IndexMap;

// ── Fix 3: Map insertion order is preserved (IndexMap) ──────────

#[test]
fn mapping_preserves_insertion_order() {
    let input = "z: 1\na: 2\nm: 3";
    let node = parse(input).unwrap();
    let map = node.value.as_mapping().unwrap();
    let keys: Vec<&MapKey> = map.keys().collect();
    assert_eq!(keys[0], &MapKey::String("z".into()));
    assert_eq!(keys[1], &MapKey::String("a".into()));
    assert_eq!(keys[2], &MapKey::String("m".into()));
}

#[test]
fn emit_mapping_preserves_insertion_order() {
    let input = "z: 1\na: 2\nm: 3";
    let node = parse(input).unwrap();
    let emitted = emit(&node);
    let lines: Vec<&str> = emitted.lines().collect();
    assert_eq!(lines[0], "z: 1");
    assert_eq!(lines[1], "a: 2");
    assert_eq!(lines[2], "m: 3");
}

#[test]
fn flow_mapping_preserves_insertion_order() {
    let input = "data: {z: 1, a: 2, m: 3}";
    let node = parse(input).unwrap();
    let map = node.value.as_mapping().unwrap();
    let inner = map[&MapKey::String("data".into())]
        .value
        .as_mapping()
        .unwrap();
    let keys: Vec<&MapKey> = inner.keys().collect();
    assert_eq!(keys[0], &MapKey::String("z".into()));
    assert_eq!(keys[1], &MapKey::String("a".into()));
    assert_eq!(keys[2], &MapKey::String("m".into()));
}

#[test]
fn round_trip_mapping_order_stable() {
    let input = "z: 1\na: 2\nm: 3\nb: 4\ny: 5\n";
    let node = parse(input).unwrap();
    // Emit and re-parse multiple times — order must be stable.
    let emitted1 = emit(&node);
    let reparsed = parse(&emitted1).unwrap();
    let emitted2 = emit(&reparsed);
    assert_eq!(emitted1, emitted2);
    assert_eq!(emitted1, input);
}

#[test]
fn nested_mapping_preserves_order() {
    let input = "\
outer:
  z: 1
  a: 2
  m: 3";
    let node = parse(input).unwrap();
    let outer = node.value.as_mapping().unwrap();
    let inner = outer[&MapKey::String("outer".into())]
        .value
        .as_mapping()
        .unwrap();
    let keys: Vec<&MapKey> = inner.keys().collect();
    assert_eq!(keys[0], &MapKey::String("z".into()));
    assert_eq!(keys[1], &MapKey::String("a".into()));
    assert_eq!(keys[2], &MapKey::String("m".into()));
}

#[test]
fn compact_mapping_in_sequence_preserves_order() {
    let input = "\
- z: 1
  a: 2
  m: 3";
    let node = parse(input).unwrap();
    let seq = node.value.as_sequence().unwrap();
    let map = seq[0].value.as_mapping().unwrap();
    let keys: Vec<&MapKey> = map.keys().collect();
    assert_eq!(keys[0], &MapKey::String("z".into()));
    assert_eq!(keys[1], &MapKey::String("a".into()));
    assert_eq!(keys[2], &MapKey::String("m".into()));
}

// ── Fix 4: Control characters are properly quoted ───────────────

#[test]
fn string_with_tab_round_trips() {
    let mut map = IndexMap::new();
    map.insert(
        MapKey::String("key".into()),
        Node::new(Value::Str("hello\tworld".into())),
    );
    let node = Node::new(Value::Map(map));
    let emitted = emit(&node);
    assert!(
        emitted.contains('"'),
        "tab-containing string should be quoted: {emitted}"
    );
    let reparsed = parse(&emitted).unwrap();
    let val = reparsed.value.as_mapping().unwrap();
    assert_eq!(
        val[&MapKey::String("key".into())].value,
        Value::Str("hello\tworld".into())
    );
}

#[test]
fn string_with_carriage_return_round_trips() {
    let mut map = IndexMap::new();
    map.insert(
        MapKey::String("key".into()),
        Node::new(Value::Str("hello\rworld".into())),
    );
    let node = Node::new(Value::Map(map));
    let emitted = emit(&node);
    assert!(
        emitted.contains('"'),
        "CR-containing string should be quoted: {emitted}"
    );
    let reparsed = parse(&emitted).unwrap();
    let val = reparsed.value.as_mapping().unwrap();
    assert_eq!(
        val[&MapKey::String("key".into())].value,
        Value::Str("hello\rworld".into())
    );
}

#[test]
fn string_with_null_byte_round_trips() {
    let mut map = IndexMap::new();
    map.insert(
        MapKey::String("key".into()),
        Node::new(Value::Str("hello\0world".into())),
    );
    let node = Node::new(Value::Map(map));
    let emitted = emit(&node);
    assert!(
        emitted.contains('"'),
        "null-byte-containing string should be quoted: {emitted}"
    );
    let reparsed = parse(&emitted).unwrap();
    let val = reparsed.value.as_mapping().unwrap();
    assert_eq!(
        val[&MapKey::String("key".into())].value,
        Value::Str("hello\0world".into())
    );
}

#[test]
fn string_with_escape_char_round_trips() {
    let mut map = IndexMap::new();
    map.insert(
        MapKey::String("key".into()),
        Node::new(Value::Str("hello\x1bworld".into())),
    );
    let node = Node::new(Value::Map(map));
    let emitted = emit(&node);
    assert!(
        emitted.contains('"'),
        "ESC-containing string should be quoted: {emitted}"
    );
    let reparsed = parse(&emitted).unwrap();
    let val = reparsed.value.as_mapping().unwrap();
    assert_eq!(
        val[&MapKey::String("key".into())].value,
        Value::Str("hello\x1bworld".into())
    );
}

#[test]
fn bare_string_without_control_chars_stays_unquoted() {
    let mut map = IndexMap::new();
    map.insert(
        MapKey::String("key".into()),
        Node::new(Value::Str("hello world".into())),
    );
    let node = Node::new(Value::Map(map));
    let emitted = emit(&node);
    assert_eq!(emitted, "key: hello world\n");
}
