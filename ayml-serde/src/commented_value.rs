//! A fully comment-preserving untyped AYML value.
//!
//! [`CommentedValue`] is `Commentable<CommentedValueKind>` — every node in the
//! tree carries optional top and inline comments, and the recursive children
//! (sequences and mappings) are themselves `CommentedValue`s.

use crate::Commentable;
use serde::de::{self, Visitor};
use serde::ser;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;

/// An untyped AYML value where every node preserves its comments.
///
/// This is a type alias for `Commentable<CommentedValueKind>`.
pub type CommentedValue = Commentable<CommentedValueKind>;

/// The kind of value inside a [`CommentedValue`].
///
/// Mirrors [`Value`](crate::Value) but uses `CommentedValue` recursively
/// so comments are preserved at every level of the tree.
///
/// Uses custom Serialize/Deserialize impls via `deserialize_any` rather than
/// `#[serde(untagged)]`, because untagged enums buffer content through serde's
/// internal `ContentDeserializer` which doesn't support `Commentable<T>`'s
/// magic struct name detection.
#[derive(Debug, Clone)]
pub enum CommentedValueKind {
    Null(()),
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Seq(Vec<CommentedValue>),
    Map(HashMap<String, CommentedValue>),
}

// ── Serialize ───────────────────────────────────────────────────────

impl Serialize for CommentedValueKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Null(()) => serializer.serialize_unit(),
            Self::Bool(b) => serializer.serialize_bool(*b),
            Self::Int(i) => serializer.serialize_i64(*i),
            Self::Float(f) => serializer.serialize_f64(*f),
            Self::Str(s) => serializer.serialize_str(s),
            Self::Seq(v) => {
                use ser::SerializeSeq;
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Self::Map(m) => {
                use ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

// ── Deserialize ─────────────────────────────────────────────────────

impl<'de> Deserialize<'de> for CommentedValueKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(CommentedValueKindVisitor)
    }
}

struct CommentedValueKindVisitor;

impl<'de> Visitor<'de> for CommentedValueKindVisitor {
    type Value = CommentedValueKind;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "any AYML value")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(CommentedValueKind::Null(()))
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
        Ok(CommentedValueKind::Bool(v))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
        Ok(CommentedValueKind::Int(v))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
        Ok(CommentedValueKind::Int(v as i64))
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
        Ok(CommentedValueKind::Float(v))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(CommentedValueKind::Str(v.to_string()))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(CommentedValueKind::Str(v))
    }

    fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element()? {
            items.push(item);
        }
        Ok(CommentedValueKind::Seq(items))
    }

    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut entries = HashMap::new();
        while let Some((key, value)) = map.next_entry()? {
            entries.insert(key, value);
        }
        Ok(CommentedValueKind::Map(entries))
    }
}

// ── PartialEq / Eq ─────────────────────────────────────────────────

impl PartialEq for CommentedValueKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null(()), Self::Null(())) => true,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => (a.is_nan() && b.is_nan()) || a == b,
            (Self::Str(a), Self::Str(b)) => a == b,
            (Self::Seq(a), Self::Seq(b)) => a == b,
            (Self::Map(a), Self::Map(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for CommentedValueKind {}

// ── Display ─────────────────────────────────────────────────────────

impl fmt::Display for CommentedValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null(()) => write!(f, "null"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Int(i) => write!(f, "{i}"),
            Self::Float(v) => {
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
            Self::Str(s) => write!(f, "{s}"),
            Self::Seq(v) => {
                write!(f, "[")?;
                for (i, item) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item.value)?;
                }
                write!(f, "]")
            }
            Self::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k}: {}", v.value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl fmt::Display for CommentedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}
