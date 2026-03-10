use serde::ser::{self, Serialize, SerializeMap, SerializeSeq};

use ayml_core::{MapKey, Node, Value};

/// Serialize a value to an AYML string.
pub fn to_string<T: Serialize>(value: &T) -> Result<String, SerError> {
    let v = value.serialize(ValueSerializer)?;
    let node = Node::new(v);
    Ok(ayml_core::emit(&node))
}

#[derive(Debug)]
pub struct SerError(String);

impl std::fmt::Display for SerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SerError {}

impl ser::Error for SerError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        SerError(msg.to_string())
    }
}

struct ValueSerializer;

impl ser::Serializer for ValueSerializer {
    type Ok = Value;
    type Error = SerError;

    type SerializeSeq = SeqSerializer;
    type SerializeTuple = SeqSerializer;
    type SerializeTupleStruct = SeqSerializer;
    type SerializeTupleVariant = SeqSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = MapSerializer;
    type SerializeStructVariant = MapSerializer;

    fn serialize_bool(self, v: bool) -> Result<Value, SerError> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Value, SerError> {
        Ok(Value::Int(v as i64))
    }

    fn serialize_i16(self, v: i16) -> Result<Value, SerError> {
        Ok(Value::Int(v as i64))
    }

    fn serialize_i32(self, v: i32) -> Result<Value, SerError> {
        Ok(Value::Int(v as i64))
    }

    fn serialize_i64(self, v: i64) -> Result<Value, SerError> {
        Ok(Value::Int(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Value, SerError> {
        Ok(Value::Int(v as i64))
    }

    fn serialize_u16(self, v: u16) -> Result<Value, SerError> {
        Ok(Value::Int(v as i64))
    }

    fn serialize_u32(self, v: u32) -> Result<Value, SerError> {
        Ok(Value::Int(v as i64))
    }

    fn serialize_u64(self, v: u64) -> Result<Value, SerError> {
        if v > i64::MAX as u64 {
            Err(SerError(format!("u64 value {v} overflows i64")))
        } else {
            Ok(Value::Int(v as i64))
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Value, SerError> {
        Ok(Value::Float(v as f64))
    }

    fn serialize_f64(self, v: f64) -> Result<Value, SerError> {
        Ok(Value::Float(v))
    }

    fn serialize_char(self, v: char) -> Result<Value, SerError> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Value, SerError> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Value, SerError> {
        Err(SerError("byte arrays are not supported in AYML".into()))
    }

    fn serialize_none(self) -> Result<Value, SerError> {
        Ok(Value::Null)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Value, SerError> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Value, SerError> {
        Ok(Value::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value, SerError> {
        Ok(Value::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value, SerError> {
        Ok(Value::String(variant.to_string()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value, SerError> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value, SerError> {
        let v = value.serialize(ValueSerializer)?;
        let mut map = std::collections::HashMap::new();
        map.insert(MapKey::String(variant.to_string()), Node::new(v));
        Ok(Value::Mapping(map))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SeqSerializer, SerError> {
        Ok(SeqSerializer {
            entries: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<SeqSerializer, SerError> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<SeqSerializer, SerError> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<SeqSerializer, SerError> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<MapSerializer, SerError> {
        Ok(MapSerializer {
            map: std::collections::HashMap::new(),
            current_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<MapSerializer, SerError> {
        Ok(MapSerializer {
            map: std::collections::HashMap::new(),
            current_key: None,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<MapSerializer, SerError> {
        Ok(MapSerializer {
            map: std::collections::HashMap::new(),
            current_key: None,
        })
    }
}

struct SeqSerializer {
    entries: Vec<Node>,
}

impl SerializeSeq for SeqSerializer {
    type Ok = Value;
    type Error = SerError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), SerError> {
        let v = value.serialize(ValueSerializer)?;
        self.entries.push(Node::new(v));
        Ok(())
    }

    fn end(self) -> Result<Value, SerError> {
        Ok(Value::Sequence(self.entries))
    }
}

impl ser::SerializeTuple for SeqSerializer {
    type Ok = Value;
    type Error = SerError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), SerError> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value, SerError> {
        SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for SeqSerializer {
    type Ok = Value;
    type Error = SerError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), SerError> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value, SerError> {
        SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleVariant for SeqSerializer {
    type Ok = Value;
    type Error = SerError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), SerError> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value, SerError> {
        SerializeSeq::end(self)
    }
}

struct MapSerializer {
    map: std::collections::HashMap<MapKey, Node>,
    current_key: Option<MapKey>,
}

impl SerializeMap for MapSerializer {
    type Ok = Value;
    type Error = SerError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), SerError> {
        let k = key.serialize(ValueSerializer)?;
        let map_key = match k {
            Value::Bool(b) => MapKey::Bool(b),
            Value::Int(i) => MapKey::Int(i),
            Value::String(s) => MapKey::String(s),
            _ => return Err(SerError("mapping keys must be bool, int, or string".into())),
        };
        self.current_key = Some(map_key);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), SerError> {
        let key = self
            .current_key
            .take()
            .ok_or_else(|| SerError("serialize_value called before serialize_key".into()))?;
        let v = value.serialize(ValueSerializer)?;
        self.map.insert(key, Node::new(v));
        Ok(())
    }

    fn end(self) -> Result<Value, SerError> {
        Ok(Value::Mapping(self.map))
    }
}

impl ser::SerializeStruct for MapSerializer {
    type Ok = Value;
    type Error = SerError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), SerError> {
        SerializeMap::serialize_key(self, key)?;
        SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Value, SerError> {
        SerializeMap::end(self)
    }
}

impl ser::SerializeStructVariant for MapSerializer {
    type Ok = Value;
    type Error = SerError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), SerError> {
        SerializeMap::serialize_key(self, key)?;
        SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Value, SerError> {
        SerializeMap::end(self)
    }
}
