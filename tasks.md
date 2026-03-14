# ayml-serde: bytes <=> types

Outside-in implementation. Start with the public API surface using `todo!()`,
then work inward until everything is functional.

## 1. Scaffold public API with `todo!()`

Stand up the six entry points in ayml-serde with `todo!()` bodies:

- `de::from_reader<R: Read, T: DeserializeOwned>(rdr: R) -> Result<T>` (std)
- `de::from_slice<T: DeserializeOwned>(bytes: &[u8]) -> Result<T>`
- `de::from_str<T: DeserializeOwned>(s: &str) -> Result<T>`
- `ser::to_string<T: Serialize>(value: &T) -> Result<String>`
- `ser::to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>>`
- `ser::to_writer<W: Write, T: Serialize>(writer: W, value: &T) -> Result<()>` (std)

Define the crate's `Error` type (implementing `serde::de::Error` +
`serde::ser::Error`) and a `Result<T>` alias. Wire up re-exports from
`lib.rs`.

## 2. Implement `Deserializer` for scalars

Implement `serde::de::Deserializer` backed by a `Scanner`. Handle all
scalar `deserialize_*` methods (bool, integers, floats, str, string,
bytes, option, unit, newtype_struct). `deserialize_any` resolves bare
scalars using AYML precedence. Sequence/map/struct/enum methods remain
`todo!()`. `from_str` becomes functional for scalar types.

## 3. Implement `Deserializer` for collections

Implement `SeqAccess` and `MapAccess` for both block and flow
collections. Handle `deserialize_seq`, `deserialize_map`,
`deserialize_struct`, `deserialize_tuple`. Indentation tracking and
comment skipping driven by the scanner on demand.

## 4. Implement `Deserializer` for enums

Implement `EnumAccess` and `VariantAccess`. Handle unit variants,
newtype variants, tuple variants, and struct variants. Covers
`deserialize_enum`.

## 5. Implement `Serializer` for scalars

Implement `serde::ser::Serializer` that writes directly to an output
buffer. Handle all scalar `serialize_*` methods (bool, integers, floats,
str, bytes, unit, none/some, newtype_struct). Sequence/map/struct methods
remain `todo!()`. `to_string` becomes functional for scalar types.

## 6. Implement `Serializer` for collections

Implement `SerializeSeq`, `SerializeMap`, `SerializeTuple`,
`SerializeStruct`, `SerializeTupleStruct`. Manage indentation state
and block-style formatting as elements are serialized.

## 7. Implement `Serializer` for enums

Implement `SerializeStructVariant` and `SerializeTupleVariant`.
Handle all enum representations.

## 8. Implement reader/writer entry points

Fill in `from_reader`, `from_slice`, `to_vec`, `to_writer` (the
non-str entry points). `from_slice`/`from_reader` convert to `&str`
(AYML is UTF-8) then delegate to `from_str`. `to_vec`/`to_writer`
delegate through `to_string`.

## 9. Test suite

Roundtrip tests (deserialize then serialize) against spec examples and
edge cases. Compatibility tests ensuring `from_str` agrees with
`ayml_core::parse` for all existing ayml-core test inputs. Serde-specific
tests: structs, enums, Options, nested types, error cases.
