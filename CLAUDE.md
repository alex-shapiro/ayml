# AYML

Simplified YAML variant built for [Ash](https://ashell.dev)

AYML is a software configuration language that looks like YAML and acts like JSON.
It is meant to be a human-friendly, cross language, Unicode based software
configuration language. Unlike YAML, it is not designed as a fully featured
data serialization framework. Most YAML features are omitted by design to avoid
unintentional or malicious misuse.

## Goals

Design goals for AYML are, in decreasing priority:

1. AYML is easy to understand
2. AYML is easy to author correctly
3. AYML is easy to deserialize into strongly typed data structures
4. AYML is expressible in the core types of dynamically typed languages
5. AYML incorporates comments as a formal part of the document structure

## Omitted

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
* multi-document files

## Crates

* `ayml` contains a serde Serializer and Deserializer.
* `ayml-core` contains a standalone parser and emitter to test spec conformance.

## Testing

AYML is tested with unit tests, integration tests, property tests, and fuzz tests. All but fuzz tests run with `cargo t`.
