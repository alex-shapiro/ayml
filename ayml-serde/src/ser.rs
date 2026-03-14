use serde::Serialize;
use serde::ser;

use crate::error::{Error, Result};

/// Serialize a `T` to an AYML string.
pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
    let mut buf = Vec::new();
    to_writer(&mut buf, value)?;
    // Safety: Serializer only writes valid UTF-8.
    Ok(unsafe { String::from_utf8_unchecked(buf) })
}

/// Serialize a `T` to an AYML byte vector.
pub fn to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    to_writer(&mut buf, value)?;
    Ok(buf)
}

/// Serialize a `T` as AYML into a writer.
pub fn to_writer<W: std::io::Write, T: Serialize>(writer: W, value: &T) -> Result<()> {
    let mut ser = Serializer::new(writer);
    value.serialize(&mut ser)?;
    Ok(())
}

// ── Serializer ──────────────────────────────────────────────────

struct Serializer<W> {
    writer: W,
}

impl<W: std::io::Write> Serializer<W> {
    fn new(writer: W) -> Self {
        Self { writer }
    }

    fn write_str(&mut self, s: &str) -> Result<()> {
        self.writer.write_all(s.as_bytes()).map_err(Error::from)
    }
}

impl<W: std::io::Write> ser::Serializer for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = ser::Impossible<(), Error>;
    type SerializeTuple = ser::Impossible<(), Error>;
    type SerializeTupleStruct = ser::Impossible<(), Error>;
    type SerializeTupleVariant = ser::Impossible<(), Error>;
    type SerializeMap = ser::Impossible<(), Error>;
    type SerializeStruct = ser::Impossible<(), Error>;
    type SerializeStructVariant = ser::Impossible<(), Error>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.write_str(if v { "true" } else { "false" })
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        self.write_str(buf.format(v))
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        self.write_str(buf.format(v))
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        if v.is_nan() {
            self.write_str("nan")
        } else if v.is_infinite() {
            if v.is_sign_positive() {
                self.write_str("inf")
            } else {
                self.write_str("-inf")
            }
        } else {
            let mut buf = ryu::Buffer::new();
            let s = buf.format_finite(v);
            // ryu may produce "1.0" or "1e0" etc. — AYML floats must have
            // a dot or exponent to distinguish from integers. ryu always
            // includes one or both, so we can emit as-is.
            self.write_str(s)
        }
    }

    fn serialize_char(self, v: char) -> Result<()> {
        let mut tmp = [0u8; 4];
        let s = v.encode_utf8(&mut tmp);
        self.serialize_str(s)
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        if v.is_empty() {
            return self.write_str(r#""""#);
        }
        if needs_quoting(v) {
            write_quoted(&mut self.writer, v).map_err(Error::from)
        } else {
            self.write_str(v)
        }
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        // Emit as a flow sequence of integers
        self.write_str("[")?;
        for (i, &byte) in v.iter().enumerate() {
            if i > 0 {
                self.write_str(", ")?;
            }
            let mut buf = itoa::Buffer::new();
            self.write_str(buf.format(byte))?;
        }
        self.write_str("]")
    }

    fn serialize_none(self) -> Result<()> {
        self.write_str("null")
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<()> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.write_str("null")
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.write_str("null")
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()> {
        todo!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        todo!()
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        todo!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        todo!()
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        todo!()
    }
}

// ── String quoting helpers ──────────────────────────────────────

/// Returns true if the string must be double-quoted in AYML output.
fn needs_quoting(s: &str) -> bool {
    // Reserved scalar values
    match s {
        "null" | "true" | "false" | "inf" | "+inf" | "-inf" | "nan" => return true,
        _ => {}
    }

    let bytes = s.as_bytes();

    // Starts with indicator character (except `-` and `:` which are
    // allowed as ns-plain-first-char when followed by ns-char)
    let first = bytes[0];
    if matches!(
        first,
        b',' | b'[' | b']' | b'{' | b'}' | b'#' | b'"' | b'\\'
    ) {
        return true;
    }

    // Starts with `- ` or `: ` (would be parsed as indicator)
    if bytes.len() >= 2 && (first == b'-' || first == b':') && bytes[1] == b' ' {
        return true;
    }

    // Looks like a number
    if looks_like_number(s) {
        return true;
    }

    // Contains characters that would break bare string parsing
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            // Control characters / non-printable
            0x00..=0x08 | 0x0B | 0x0C | 0x0E..=0x1F | 0x7F => return true,
            // Line breaks — bare strings are single-line
            b'\n' | b'\r' => return true,
            // Embedded quotes need escaping
            b'"' | b'\\' => return true,
            // `: ` mid-string would be parsed as mapping indicator
            b':' if i + 1 < bytes.len() && bytes[i + 1] == b' ' => return true,
            // ` #` would start a comment
            b'#' if i > 0 && bytes[i - 1] == b' ' => return true,
            _ => {}
        }
    }

    // Trailing whitespace would be stripped
    if bytes.last().is_some_and(|&b| b == b' ' || b == b'\t') {
        return true;
    }

    // Leading whitespace
    if first == b' ' || first == b'\t' {
        return true;
    }

    false
}

/// Check if a string looks like it would be parsed as a number.
fn looks_like_number(s: &str) -> bool {
    let s = s
        .strip_prefix('+')
        .or_else(|| s.strip_prefix('-'))
        .unwrap_or(s);
    if s.is_empty() {
        return false;
    }

    // 0b, 0o, 0x prefixes
    if s.starts_with("0b") || s.starts_with("0o") || s.starts_with("0x") {
        return true;
    }

    // All digits → integer
    if s.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    // Contains dot or e/E with digits → float
    if s.contains('.') || s.contains('e') || s.contains('E') {
        // Quick check: starts with digit
        if s.as_bytes()[0].is_ascii_digit() {
            return true;
        }
    }

    false
}

/// Write a double-quoted AYML string with escaping.
fn write_quoted<W: std::io::Write>(w: &mut W, s: &str) -> std::io::Result<()> {
    w.write_all(b"\"")?;
    for ch in s.chars() {
        match ch {
            '\0' => w.write_all(b"\\0")?,
            '\x07' => w.write_all(b"\\a")?,
            '\x08' => w.write_all(b"\\b")?,
            '\t' => w.write_all(b"\\t")?,
            '\n' => w.write_all(b"\\n")?,
            '\x0B' => w.write_all(b"\\v")?,
            '\x0C' => w.write_all(b"\\f")?,
            '\r' => w.write_all(b"\\r")?,
            '\x1B' => w.write_all(b"\\e")?,
            '"' => w.write_all(b"\\\"")?,
            '\\' => w.write_all(b"\\\\")?,
            c if c.is_control() => {
                let cp = c as u32;
                if cp <= 0xFF {
                    write!(w, "\\x{cp:02x}")?;
                } else if cp <= 0xFFFF {
                    write!(w, "\\u{cp:04x}")?;
                } else {
                    write!(w, "\\U{cp:08x}")?;
                }
            }
            c => {
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);
                w.write_all(s.as_bytes())?;
            }
        }
    }
    w.write_all(b"\"")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool() {
        assert_eq!(to_string(&true).unwrap(), "true");
        assert_eq!(to_string(&false).unwrap(), "false");
    }

    #[test]
    fn test_integers() {
        assert_eq!(to_string(&0i32).unwrap(), "0");
        assert_eq!(to_string(&42i32).unwrap(), "42");
        assert_eq!(to_string(&-17i64).unwrap(), "-17");
        assert_eq!(to_string(&255u8).unwrap(), "255");
        assert_eq!(to_string(&u64::MAX).unwrap(), "18446744073709551615");
    }

    #[test]
    fn test_floats() {
        assert_eq!(to_string(&3.14f64).unwrap(), "3.14");
        assert_eq!(to_string(&-0.5f64).unwrap(), "-0.5");
        assert_eq!(to_string(&f64::INFINITY).unwrap(), "inf");
        assert_eq!(to_string(&f64::NEG_INFINITY).unwrap(), "-inf");
        assert_eq!(to_string(&f64::NAN).unwrap(), "nan");
    }

    #[test]
    fn test_float_whole_number() {
        // ryu emits "1.0" for 1.0f64 — must not be confused with integer
        let s = to_string(&1.0f64).unwrap();
        assert!(s.contains('.') || s.contains('e') || s.contains('E'));
    }

    #[test]
    fn test_string_bare() {
        assert_eq!(to_string(&"hello").unwrap(), "hello");
        assert_eq!(to_string(&"hello world").unwrap(), "hello world");
        assert_eq!(
            to_string(&"https://example.com").unwrap(),
            "https://example.com"
        );
    }

    #[test]
    fn test_string_empty() {
        assert_eq!(to_string(&"").unwrap(), r#""""#);
    }

    #[test]
    fn test_string_needs_quoting() {
        // Reserved words
        assert_eq!(to_string(&"null").unwrap(), r#""null""#);
        assert_eq!(to_string(&"true").unwrap(), r#""true""#);
        assert_eq!(to_string(&"false").unwrap(), r#""false""#);
        assert_eq!(to_string(&"inf").unwrap(), r#""inf""#);
        assert_eq!(to_string(&"nan").unwrap(), r#""nan""#);

        // Looks like number
        assert_eq!(to_string(&"42").unwrap(), r#""42""#);
        assert_eq!(to_string(&"3.14").unwrap(), r#""3.14""#);
        assert_eq!(to_string(&"0xFF").unwrap(), r#""0xFF""#);
    }

    #[test]
    fn test_string_escapes() {
        assert_eq!(to_string(&"line\nbreak").unwrap(), r#""line\nbreak""#);
        assert_eq!(to_string(&"say \"hi\"").unwrap(), r#""say \"hi\"""#);
        assert_eq!(to_string(&"back\\slash").unwrap(), r#""back\\slash""#);
        assert_eq!(to_string(&"\x00null").unwrap(), r#""\0null""#);
    }

    #[test]
    fn test_string_indicator_chars() {
        // Starts with indicator
        assert_eq!(to_string(&"[array").unwrap(), r#""[array""#);
        assert_eq!(to_string(&"{map").unwrap(), r#""{map""#);
        assert_eq!(to_string(&"#comment").unwrap(), "\"#comment\"");

        // Contains `: ` mid-string
        assert_eq!(to_string(&"key: value").unwrap(), r#""key: value""#);

        // Contains ` #` (comment)
        assert_eq!(to_string(&"value #comment").unwrap(), r#""value #comment""#);
    }

    #[test]
    fn test_string_leading_trailing_whitespace() {
        assert_eq!(to_string(&" leading").unwrap(), r#"" leading""#);
        assert_eq!(to_string(&"trailing ").unwrap(), r#""trailing ""#);
    }

    #[test]
    fn test_null() {
        assert_eq!(to_string::<()>(&()).unwrap(), "null");
    }

    #[test]
    fn test_option() {
        assert_eq!(to_string(&None::<i32>).unwrap(), "null");
        assert_eq!(to_string(&Some(42)).unwrap(), "42");
        assert_eq!(to_string(&Some("hello")).unwrap(), "hello");
    }

    #[test]
    fn test_newtype_struct() {
        #[derive(serde::Serialize)]
        struct Meters(f64);
        assert_eq!(to_string(&Meters(3.5)).unwrap(), "3.5");
    }

    #[test]
    fn test_unit_struct() {
        #[derive(serde::Serialize)]
        struct Marker;
        assert_eq!(to_string(&Marker).unwrap(), "null");
    }

    #[test]
    fn test_unit_variant() {
        #[derive(serde::Serialize)]
        enum Color {
            Red,
            Green,
        }
        assert_eq!(to_string(&Color::Red).unwrap(), "Red");
        assert_eq!(to_string(&Color::Green).unwrap(), "Green");
    }

    #[test]
    fn test_char() {
        assert_eq!(to_string(&'a').unwrap(), "a");
        assert_eq!(to_string(&'\n').unwrap(), r#""\n""#);
        assert_eq!(to_string(&'"').unwrap(), r#""\"""#);
    }

    #[test]
    fn test_bytes() {
        let bytes: &[u8] = &[1, 2, 3];
        assert_eq!(
            to_string(&serde_bytes::Bytes::new(bytes)).unwrap(),
            "[1, 2, 3]"
        );
    }

    #[test]
    fn test_to_vec() {
        let v = to_vec(&42i32).unwrap();
        assert_eq!(v, b"42");
    }

    #[test]
    fn test_to_writer() {
        let mut buf = Vec::new();
        to_writer(&mut buf, &"hello").unwrap();
        assert_eq!(buf, b"hello");
    }
}
