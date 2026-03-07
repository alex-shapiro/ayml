# AYML

Simplified YAML variant built for [Ash](https://ashell.dev)

AYML is a software configuration language that looks like YAML and acts like JSON.
It is meant to be a human-friendly, cross language, Unicode based software
configuration language. Unlike YAML, it is not designed as a fully featured
data serialization framework. There are no references types, unordered sets,
duplicate nodes, document prefixes, or other complex features supported by YAML.

## Explicit Nongoals

* YES/NO for booleans
* tags
* structures (anchors, aliases, `---` separators)
* unordered sets
* duplicate keys
* JSON compatibility quirks
* empty nodes
* `?` indicators
* directives (except an optional `%AYML 1.0`)
* document prefixes
* multiple documents per file
