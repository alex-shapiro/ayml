//! Comment-preserving wrapper for AYML values.

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;

/// Magic struct name used to signal comment-aware (de)serialization.
pub(crate) const COMMENTED_STRUCT: &str = "__commented__";
pub(crate) const FIELD_TOP_COMMENT: &str = "__top_comment__";
pub(crate) const FIELD_INLINE_COMMENT: &str = "__inline_comment__";
pub(crate) const FIELD_VALUE: &str = "__value__";

/// A wrapper that preserves AYML comments through serde round-trips.
///
/// When used as a field type in a struct deserialized from AYML, comments
/// adjacent to the value are captured:
///
/// ```yaml
/// # top comment
/// port: 8080  # inline comment
/// ```
///
/// ```rust,ignore
/// #[derive(Deserialize)]
/// struct Config {
///     port: Commented<u16>,
/// }
/// let c: Config = ayml::from_str(input)?;
/// assert_eq!(c.port.value, 8080);
/// assert_eq!(c.port.top_comment.as_deref(), Some("top comment"));
/// assert_eq!(c.port.inline_comment.as_deref(), Some("inline comment"));
/// ```
///
/// When serialized back to AYML, comments are emitted in their original
/// positions. For non-AYML serializers, the comments are serialized as
/// struct fields and are losslessly preserved.
#[derive(Debug, Clone, PartialEq)]
pub struct Commented<T> {
    /// Comment line(s) preceding the value. Multi-line comments are
    /// joined with `\n`. Does not include the `# ` prefix.
    pub top_comment: Option<String>,
    /// Comment on the same line after the value. Does not include `# `.
    pub inline_comment: Option<String>,
    /// The wrapped value.
    pub value: T,
}

impl<T> Commented<T> {
    /// Wrap a value with no comments.
    pub fn new(value: T) -> Self {
        Self {
            top_comment: None,
            inline_comment: None,
            value,
        }
    }
}

impl<T: Default> Default for Commented<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Serialize> Serialize for Commented<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct(COMMENTED_STRUCT, 3)?;
        s.serialize_field(FIELD_TOP_COMMENT, &self.top_comment)?;
        s.serialize_field(FIELD_INLINE_COMMENT, &self.inline_comment)?;
        s.serialize_field(FIELD_VALUE, &self.value)?;
        s.end()
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Commented<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        const FIELDS: &[&str] = &[FIELD_TOP_COMMENT, FIELD_INLINE_COMMENT, FIELD_VALUE];
        deserializer.deserialize_struct(COMMENTED_STRUCT, FIELDS, CommentedVisitor(PhantomData))
    }
}

struct CommentedVisitor<T>(PhantomData<T>);

impl<'de, T: Deserialize<'de>> Visitor<'de> for CommentedVisitor<T> {
    type Value = Commented<T>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a commented value")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut top_comment: Option<String> = None;
        let mut inline_comment: Option<String> = None;
        let mut value: Option<T> = None;

        while let Some(key) = map.next_key::<&str>()? {
            match key {
                FIELD_TOP_COMMENT => {
                    top_comment = map.next_value()?;
                }
                FIELD_INLINE_COMMENT => {
                    inline_comment = map.next_value()?;
                }
                FIELD_VALUE => {
                    value = Some(map.next_value()?);
                }
                _ => {
                    // Skip unknown fields
                    map.next_value::<de::IgnoredAny>()?;
                }
            }
        }

        let value = value.ok_or_else(|| de::Error::missing_field(FIELD_VALUE))?;
        Ok(Commented {
            top_comment,
            inline_comment,
            value,
        })
    }
}

#[cfg(feature = "schemars")]
impl<T: schemars::JsonSchema> schemars::JsonSchema for Commented<T> {
    fn schema_name() -> String {
        T::schema_name()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        T::schema_id()
    }

    fn json_schema(generator: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        T::json_schema(generator)
    }

    fn is_referenceable() -> bool {
        T::is_referenceable()
    }
}
