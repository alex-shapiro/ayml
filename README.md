# AYML

> [!WARNING]
> Status: Not Ready For Production Use

Simplified YAML variant built for [Ash](https://ashell.dev)

AYML is a software configuration language that looks like YAML and acts like JSON.
It is meant to be a human-friendly, cross language, Unicode based software
configuration language. Unlike YAML, it is not designed as a fully featured
data serialization framework. Most YAML features are omitted by design to avoid
unintentional or malicious misuse.

## Example

It look a lot like YAML:

```ayml
# yaml-language-server: $schema=https://hub.ashell.dev/schemas/policy/v1.json
schema_version: 1
dependencies: 
  # default macOS permissions
  ash/base-macos: ^0
  # allow git command line tools
  ash/git: ^0
  # Ash internal team policy, hosted locally
  ash/internal:
    path: ~/.ash/policies/internal.ayml
files:
  rules:
    # full access to everything in the current directory
    - path: ./**
network:
  rules:
    - host: **.crates.io # crate publishing
      ports:
        - 443
```

## Goals

AYML's design goals are, in decreasing priority:

1. it is easy to read and understand
2. it is easy to write correctly (and hard to write incorrectly)
3. it deserializes naturally into strongly typed data structures
4. it deserializes naturally into the core types of dynamically typed languages
5. it incorporates comments as a formal part of the document structure

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

#### Fuzzing

To run fuzz tests:

```bash
rustup toolchain install nightly
cargo install cargo-fuzz
cd ayml/fuzz

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

#### Benchmarking

```sh
# benchmark ayml
cargo bench

# benchmark ayml against serde_json
cargo bench --bench serde_comparison
```

All tests performed on a Macbook Pro M4 Max.
Deserialization benchmarks are for `typed` variations (`value` variations are roughly 60% as fast). 

|  Benchmark  | Deserialize | Serialize |
|-------------|-------------|-----------|
| flat        | 106 MiB/s   | 364 MiB/s |
| nested      | 106 MiB/s   | 434 MiB/s |
| seq_of_maps | 101 MiB/s   | 483 MiB/s |
| strings     | 222 MiB/s   | 602 MiB/s |
| large_50    | 104 MiB/s   | 565 MiB/s |

Based on comparative benchmarks, if you are looking for maximal performance then you should consider `serde_json`. It deserializes 6-7x faster and serializes 2-3x faster than AYML. AYML performance is adequate for typical config use cases, where serialization performance is not a bottleneck.

## Acknowledgements

* The AYML spec doc is derived in large part from the [YAML specification](https://yaml.org/spec).
* The AYML serde implementation is heavily influenced by [serde_json](https://github.com/serde-rs/json)
