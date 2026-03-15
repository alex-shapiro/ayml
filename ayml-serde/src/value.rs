//! An untyped AYML value, analogous to `serde_json::Value`.

use indexmap::IndexMap;
use serde::Serialize;
use serde::de::{self, Deserializer, Visitor};
use std::fmt;

/// An untyped AYML value that can represent any AYML document.
///
/// Covers all six AYML value kinds: null, boolean, integer, float,
/// string, sequence, and mapping.
///
/// Uses a hand-written `Deserialize` impl (like `serde_json::Value`)
/// that calls `deserialize_any` directly instead of `#[serde(untagged)]`,
/// avoiding the intermediate `Content` buffering that would otherwise
/// duplicate every node in the tree during deserialization.
///
/// # Equality
///
/// `PartialEq` is NaN-aware: two `Float(NaN)` values compare equal,
/// matching the expectation that a roundtripped NaN should equal itself.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Value {
    /// AYML `null`.
    Null,
    /// AYML `true` or `false`.
    Bool(bool),
    /// AYML integer (64-bit signed).
    Int(i64),
    /// AYML float (64-bit IEEE 754).
    Float(f64),
    /// AYML string (bare or double-quoted).
    Str(String),
    /// AYML sequence (`- ` items or `[...]` flow).
    Seq(Vec<Value>),
    /// AYML mapping (`key: value` pairs or `{...}` flow).
    Map(IndexMap<String, Value>),
}

// ── Deserialize ─────────────────────────────────────────────────────

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ValueVisitor)
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "any AYML value")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(v))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
        Ok(Value::Int(v))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
        let i = i64::try_from(v)
            .map_err(|_| de::Error::custom(format_args!("u64 value {v} exceeds i64::MAX")))?;
        Ok(Value::Int(i))
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
        Ok(Value::Float(v))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(Value::Str(v.to_owned()))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(Value::Str(v))
    }

    fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element()? {
            items.push(item);
        }
        Ok(Value::Seq(items))
    }

    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut entries = IndexMap::new();
        while let Some((key, value)) = map.next_entry()? {
            entries.insert(key, value);
        }
        Ok(Value::Map(entries))
    }
}

// ── PartialEq / Eq ─────────────────────────────────────────────────

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a.is_nan() && b.is_nan()) || a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Seq(a), Value::Seq(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            _ => false,
        }
    }
}

// Note: `Eq` is intentionally not implemented because `Value` can contain
// `Float(f64)` and floats do not satisfy the `Eq` contract (transitivity).
// The custom `PartialEq` treats NaN == NaN for testing convenience.

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(v) => {
                if v.is_nan() {
                    write!(f, "nan")
                } else if v.is_infinite() {
                    if v.is_sign_positive() {
                        write!(f, "inf")
                    } else {
                        write!(f, "-inf")
                    }
                } else {
                    write!(f, "{v}")
                }
            }
            Value::Str(s) => write!(f, "{s}"),
            Value::Seq(v) => {
                write!(f, "[")?;
                for (i, item) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Value::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k}: {v}")?;
                }
                write!(f, "}}")
            }
        }
    }
}
