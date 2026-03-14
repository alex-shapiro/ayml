# ayml-serde

## 1. Deserializer: collections

Implement `SeqAccess` and `MapAccess` for both block and flow
collections. Handle `deserialize_seq`, `deserialize_map`,
`deserialize_struct`, `deserialize_tuple`. Indentation tracking and
comment skipping driven by the `Read` trait on demand.

## 2. Deserializer: enums

Implement `EnumAccess` and `VariantAccess`. Handle unit variants,
newtype variants, tuple variants, and struct variants. Covers
`deserialize_enum`.

## 3. Serializer: scalars

Implement `serde::ser::Serializer` that writes directly to an output
buffer. Handle all scalar `serialize_*` methods (bool, integers, floats,
str, bytes, unit, none/some, newtype_struct). Collection/enum methods
remain `todo!()`. `to_string` becomes functional for scalar types.

## 4. Serializer: collections

Implement `SerializeSeq`, `SerializeMap`, `SerializeTuple`,
`SerializeStruct`, `SerializeTupleStruct`. Manage indentation state
and block-style formatting as elements are serialized.

## 5. Serializer: enums

Implement `SerializeStructVariant` and `SerializeTupleVariant`.
Handle all enum representations.

## 6. Serializer: `to_vec` and `to_writer`

`to_vec` wraps `to_string` into bytes. `to_writer` writes directly
to `W: io::Write` instead of building a String.

## 7. Test suite

Roundtrip tests (deserialize then serialize) against spec examples and
edge cases. Serde-specific tests: structs, enums, Options, nested types,
error cases.
