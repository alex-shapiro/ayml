# AYML

Simplified YAML variant built for [Ash](https://ashell.dev)

AYML is a software configuration language that looks like YAML and acts like JSON.
It is meant to be a human-friendly, cross language, Unicode based software
configuration language. Unlike YAML, it is not designed as a fully featured
data serialization framework. There are no references types, unordered sets,
duplicate nodes, document prefixes, or other complex features supported by YAML.

## Explicit Nongoals

* bool value must be true or false (not YES/NO, avoid Norway problem)
* no tags
* no structures (anchors, aliases, `---` separators)
* no unordered Sets
* no duplicate keys
* no JSON compatibility quirks
* no empty nodes
* no `?` indicators
* no directives except one: `%AYML 1.0` (this directive is optional
* no document prefixes
* multiple documents per file
* ...etc
