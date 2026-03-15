# AYML

Simplified YAML variant built for [Ash](https://ashell.dev)

AYML is a software configuration language that looks like YAML and acts like JSON.
It is meant to be a human-friendly, cross language, Unicode based software
configuration language. Unlike YAML, it is not designed as a fully featured
data serialization framework. There are no references types, unordered sets,
duplicate nodes, document prefixes, or other complex features supported by YAML.

## Goals

Design goals for AYML are, in decreasing priority:

1. AYML is easy to understand
1. AYML is easy to author correctly
1. AYML is easy to deserialize into strongly typed data structures
1. AYML is expressible in the core types of dynamically typed languages
1. AYML incorporates comments as a formal part of the document structure

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
* multi-document files
