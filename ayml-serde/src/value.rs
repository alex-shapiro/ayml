//! An untyped AYML value, analogous to `serde_json::Value`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// An untyped AYML value that can represent any AYML document.
///
/// Covers all six AYML value kinds: null, boolean, integer, float,
/// string, sequence, and mapping. Uses `#[serde(untagged)]` so it
/// deserializes naturally from any AYML input via `deserialize_any`.
///
/// # Equality
///
/// `PartialEq` is NaN-aware: two `Float(NaN)` values compare equal,
/// matching the expectation that a roundtripped NaN should equal itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    Map(HashMap<String, Value>),
}

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

impl Eq for Value {}

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
