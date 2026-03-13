use serde::Deserialize;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};

use ayml_core::{Error as AymlError, MapKey, Node, Value};

/// Deserialize a type from an AYML string.
pub fn from_str<'de, T: Deserialize<'de>>(s: &'de str) -> Result<T, AymlError> {
    let node = ayml_core::parse(s)?;
    let deserializer = ValueDeserializer::new(&node.value);
    T::deserialize(deserializer).map_err(|e| AymlError {
        kind: ayml_core::ErrorKind::Expected(e.to_string()),
        span: ayml_core::Span::point(0),
        line: 0,
        column: 0,
    })
}

struct ValueDeserializer<'a> {
    value: &'a Value,
}

impl<'a> ValueDeserializer<'a> {
    fn new(value: &'a Value) -> Self {
        Self { value }
    }
}

#[derive(Debug)]
struct DeError(String);

impl std::fmt::Display for DeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DeError {}

impl de::Error for DeError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        DeError(msg.to_string())
    }
}

impl<'de, 'a> de::Deserializer<'de> for ValueDeserializer<'a> {
    type Error = DeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Null => visitor.visit_unit(),
            Value::Bool(b) => visitor.visit_bool(*b),
            Value::Int(i) => visitor.visit_i64(*i),
            Value::Float(f) => visitor.visit_f64(*f),
            Value::Str(s) => visitor.visit_str(s),
            Value::Seq(seq) => {
                let access = SeqDeserializer::new(seq);
                visitor.visit_seq(access)
            }
            Value::Map(map) => {
                let access = MapDeserializer::new(map);
                visitor.visit_map(access)
            }
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

struct SeqDeserializer<'a> {
    iter: std::slice::Iter<'a, Node>,
}

impl<'a> SeqDeserializer<'a> {
    fn new(seq: &'a [Node]) -> Self {
        Self { iter: seq.iter() }
    }
}

impl<'de, 'a> SeqAccess<'de> for SeqDeserializer<'a> {
    type Error = DeError;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        match self.iter.next() {
            Some(node) => seed
                .deserialize(ValueDeserializer::new(&node.value))
                .map(Some),
            None => Ok(None),
        }
    }
}

struct MapDeserializer<'a> {
    iter: indexmap::map::Iter<'a, MapKey, Node>,
    current_value: Option<&'a Node>,
}

impl<'a> MapDeserializer<'a> {
    fn new(map: &'a indexmap::IndexMap<MapKey, Node>) -> Self {
        Self {
            iter: map.iter(),
            current_value: None,
        }
    }
}

impl<'de, 'a> MapAccess<'de> for MapDeserializer<'a> {
    type Error = DeError;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        match self.iter.next() {
            Some((key, value)) => {
                self.current_value = Some(value);
                let key_value = match key {
                    MapKey::Bool(b) => Value::Bool(*b),
                    MapKey::Int(i) => Value::Int(*i),
                    MapKey::String(s) => Value::Str(s.clone()),
                };
                seed.deserialize(ValueDeserializer::new(&key_value))
                    .map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        let node = self
            .current_value
            .take()
            .ok_or_else(|| DeError("expected value".into()))?;
        seed.deserialize(ValueDeserializer::new(&node.value))
    }
}
