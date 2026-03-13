# PR Review: `rust-lib` — AYML Rust Workspace

~5500 lines across 4 crates (`ayml-core`, `ayml-serde`, `ayml-ffi`, `ayml` umbrella), test fixtures, and spec updates.

## High Priority

### 1. `i64::MIN` cannot be parsed as integer — `ayml-core/src/parser/grammar.rs:672`
The parser splits negative integers into sign + magnitude. For `-9223372036854775808`, the magnitude `9223372036854775808` overflows `i64`, so it falls through to a string. Fix: parse the full string including the `-` sign as a single `i64`, or special-case MIN.

### 2. FFI `ayml_node_string` — pointer obtained before move — `ayml-ffi/src/lib.rs:200-206`
`cs.as_ptr()` is called *before* `doc.strings.push(cs)` moves `cs`. This happens to work because `CString` stores data on the heap, but it's a latent UB risk. The idiomatic fix:
```rust
doc.strings.push(cs);
doc.strings.last().unwrap().as_ptr()
```

### 3. Tab indentation silently accepted (spec violation)
The spec says tabs MUST NOT be used for indentation, and a `TabIndent` error variant exists in `error.rs:37`, but the parser never checks for or rejects tabs in indentation. Tab-indented input is silently parsed.

### 4. `IntegerOverflow` error variant is dead code — `error.rs:51`
Defined but never produced. Overflowing integers silently become strings.

## Medium Priority

### 5. `process_escape_char` treats EOF as literal backslash — `grammar.rs:516`
```rust
Some('\\') | None => result.push('\\'),
```
If `\` is the last character in a quoted string (malformed input), it silently emits `\` instead of erroring.

### 6. FFI lacks sequence/map element accessors — `ayml-ffi/src/lib.rs`
Exposes `ayml_node_seq_len` and `ayml_node_map_len` but provides no way to access individual elements, making the FFI unusable for non-scalar structures.

### 7. Zero tests for `ayml-serde`
No struct deserialization, serialization, `Option<T>`, enum handling, or error message tests.

### 8. Inconsistent error types — `de.rs`, `ser.rs`
`serde::from_str` returns `AymlError` while `serde::to_string` returns `SerError`. Serde deserialization errors also lose all location info (hardcoded to `span: Span::point(0), line: 0, col: 0`).

## Low Priority

### 9. Comment indent parameter unused — `grammar.rs:127`
`parse_comment_block(&mut self, _n: usize)` — the `_n` parameter is never used to validate indentation.

### 10. `UnexpectedEof` as hard error may suppress backtracking — `grammar.rs:26`
Marks `UnexpectedEof` as non-backtrackable, which can produce confusing errors for inputs like a lone `"`.

### 11. No `indexmap` re-export from umbrella crate — `ayml/src/lib.rs`
Users need `IndexMap` to work with `Value::Map` but must add `indexmap` as a separate dependency.

### 12. Missing convenience impls
No `From<&str>` for `MapKey`, no `node.as_str()` delegation methods.

## Test Coverage Gaps

- No tests for tab-in-indentation rejection
- No tests for `i64::MIN` parsing
- No serde integration tests
- No tests for CR or CRLF line endings
- No tests for non-printable character rejection
- `IntegerOverflow` error variant never tested (never produced)
- No tests for deeply nested block sequences
- No tests for the `ayml` umbrella crate

## What Looks Good

- Parser architecture with save/restore checkpointing is clean
- Proptest integration is valuable and has already caught regressions
- The emitter correctly handles all value types with proper quoting
- Error reporting with spans, line/column is well-structured
- FFI memory management model (document owns all allocations) is sound in principle
- Good test coverage of core parsing (scalars, collections, comments, spec examples)
