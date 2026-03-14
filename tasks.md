# ayml-serde: bytes <=> types

## Phase 1: Extract shared primitives from ayml-core

### 1. Make `Scanner` public
Expose `Scanner` as a public type from ayml-core so ayml-serde can use it directly.
Currently `pub` within the parser module but not re-exported from the crate root.

### 2. Extract scalar resolution into standalone functions
Factor `parse_bare_scalar`, `try_parse_int`, `try_parse_float`, and the
null/bool keyword matching out of `Parser` into free functions that operate on
`&mut Scanner` and return a type-tag + borrowed slice (not `Value`/`Node`).
`Parser` calls these new functions internally so behavior is unchanged.

### 3. Extract structural parsing helpers
Factor out the indentation/structure detection logic — `skip_blank_lines`,
`skip_block_gaps`, comment skipping, and the "what comes next?" peek logic
(is it `- `, `key:`, `[`, `{`, or a scalar?) — into free functions on
`&mut Scanner` + indent level. `Parser` calls these internally.

### 4. Extract double-quoted and triple-quoted string parsing
Factor `parse_double_quoted` and `parse_triple_quoted` out of `Parser` into
free functions on `&mut Scanner` that return `String` (not `Node`). `Parser`
wraps the result in `Node::new(Value::Str(...))`.

### 5. Make emitter primitives public
Expose `needs_quoting`, `emit_float`, `emit_string`, `emit_map_key`, and
`emit_indent` as public functions from ayml-core so the serde serializer can
write AYML directly without building a `Value` tree.

### 6. Extend `ErrorKind` for serde
Add `Message(String)` variant to `ErrorKind`. Implement `serde::de::Error`
and `serde::ser::Error` for `Error` behind a `serde` cargo feature flag on
ayml-core.

## Phase 2: Implement ayml-serde

### 7. Implement streaming `Deserializer`
Implement `serde::de::Deserializer` directly over `Scanner` using the
extracted primitives from phase 1. Includes `SeqAccess`, `MapAccess`, and
`EnumAccess` impls. Drives parsing on-demand as the visitor requests types.
Public API: `ayml_serde::from_str<T: DeserializeOwned>(s: &str) -> Result<T, Error>`.

### 8. Implement streaming `Serializer`
Implement `serde::ser::Serializer` that writes AYML bytes directly using the
extracted emitter primitives. Manages indentation and block/flow style as
serde calls `serialize_*` methods. Includes `SerializeSeq`, `SerializeMap`,
`SerializeStruct`, etc.
Public API: `ayml_serde::to_string<T: Serialize>(value: &T) -> Result<String, Error>`.

### 9. Test suite
Roundtrip tests (deserialize then serialize) against spec examples and edge
cases. Compatibility tests ensuring `from_str` produces the same results as
`ayml_core::parse` for all existing ayml-core test inputs. Serde-specific
tests: structs, enums, Options, nested types, error cases.
