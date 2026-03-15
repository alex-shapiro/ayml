# AYML

> Status: Not Ready For Production Use

Simplified YAML variant built for [Ash](https://ashell.dev)

AYML is a software configuration language that looks like YAML and acts like JSON.
It is meant to be a human-friendly, cross language, Unicode based software
configuration language. Unlike YAML, it is not designed as a fully featured
data serialization framework. There are no references types, unordered sets,
duplicate nodes, document prefixes, or other complex features supported by YAML.

## Goals

The design goals for AYML are, in decreasing priority:

1. AYML should be easy to understand
1. AYML should be easy to author correctly
1. AYML should be easy to deserialize into strongly typed data structures
1. AYML should be expressible in the core types of dynamically typed languages
1. AYML should incorporate comments as a formal part of the document structure

## Nongoals

* YES/NO for booleans
* tags
* structures (anchors, aliases, `---` separators)
* unordered sets
* duplicate keys
* JSON compatibility quirks
* empty nodes
* `?` indicators
* directives
* document prefixes
* mult-document files

## Implementation

### Fuzzing

To run fuzz tests:

```bash
rustup toolchain install nightly
cargo install cargo-fuzz
cd ayml-serde/fuzz

# Run the deserialize fuzzer
cargo +nightly fuzz run fuzz_deserialize

# Run the roundtrip fuzzer
cargo +nightly fuzz run fuzz_roundtrip

# Limit to 5 minutes
cargo +nightly fuzz run fuzz_deserialize -- -max_total_time=300
```

Fuzz test crashes are saved to `fuzz/artifacts/<target>/`. Reproduce with:

```bash
cargo +nightly fuzz run fuzz_deserialize fuzz/artifacts/fuzz_deserialize/crash-<hash>
```
