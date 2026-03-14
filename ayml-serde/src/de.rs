use serde::de::{self, DeserializeOwned, Visitor};

use crate::error::{Error, Result};
use crate::read::{IoRead, Read, StrRead};

/// Deserialize a `T` from a string of AYML text.
///
/// The bound `T: Deserialize<'a>` (rather than `DeserializeOwned`) allows
/// zero-copy deserialization of borrowed types like `&'a str`.
pub fn from_str<'a, T: serde::Deserialize<'a>>(s: &'a str) -> Result<T> {
    let mut de = Deserializer::from_str(s);
    let value = T::deserialize(&mut de)?;
    de.end()?;
    Ok(value)
}

/// Deserialize a `T` from a slice of AYML bytes.
///
/// AYML is UTF-8; this validates the input and then deserializes.
/// Supports borrowing from the input slice (`T: Deserialize<'a>`).
pub fn from_slice<'a, T: serde::Deserialize<'a>>(bytes: &'a [u8]) -> Result<T> {
    let s =
        std::str::from_utf8(bytes).map_err(|e| Error::Message(format!("invalid UTF-8: {e}")))?;
    from_str(s)
}

/// Deserialize a `T` from an AYML reader.
///
/// Data is read lazily from the reader as the deserializer advances.
/// The deserialized value cannot borrow from the input; use [`from_str`]
/// or [`from_slice`] for zero-copy deserialization.
pub fn from_reader<R: std::io::Read, T: DeserializeOwned>(rdr: R) -> Result<T> {
    let mut de = Deserializer::from_reader(rdr);
    let value = T::deserialize(&mut de)?;
    de.end()?;
    Ok(value)
}

// ── Parsing context ──────────────────────────────────────────────

/// Block vs flow parsing context, mirroring ayml-core's grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Context {
    Block,
    Flow,
}

// ── Deserializer ─────────────────────────────────────────────────

pub(crate) struct Deserializer<R> {
    read: R,
    /// Scratch buffer for building strings that require escape processing.
    /// Bare strings that need no processing can borrow directly from input
    /// (zero-copy path, to be wired up in a future pass).
    #[allow(dead_code)]
    scratch: Vec<u8>,
}

impl<'a> Deserializer<StrRead<'a>> {
    fn from_str(s: &'a str) -> Self {
        Self {
            read: StrRead::new(s),
            scratch: Vec::new(),
        }
    }
}

impl<R: std::io::Read> Deserializer<IoRead<R>> {
    fn from_reader(rdr: R) -> Self {
        Self {
            read: IoRead::new(rdr),
            scratch: Vec::new(),
        }
    }
}

// ── Character-level helpers ──────────────────────────────────────

impl<'de, R: Read<'de>> Deserializer<R> {
    fn peek(&mut self) -> Result<Option<char>> {
        let off = self.read.offset();
        self.read.fill_to(off + 4)?;
        let input = self.read.input();
        if off >= input.len() {
            Ok(None)
        } else {
            Ok(input[off..].chars().next())
        }
    }

    fn peek_nth(&mut self, n: usize) -> Result<Option<char>> {
        let off = self.read.offset();
        self.read.fill_to(off + (n + 1) * 4)?;
        let input = self.read.input();
        if off >= input.len() {
            Ok(None)
        } else {
            Ok(input[off..].chars().nth(n))
        }
    }

    fn advance(&mut self) -> Result<Option<char>> {
        let ch = self.peek()?;
        if let Some(c) = ch {
            let off = self.read.offset();
            self.read.set_offset(off + c.len_utf8());
        }
        Ok(ch)
    }

    fn is_eof(&mut self) -> Result<bool> {
        Ok(self.peek()?.is_none())
    }

    fn is_break_or_eof(&mut self) -> Result<bool> {
        match self.peek()? {
            None | Some('\n' | '\r') => Ok(true),
            _ => Ok(false),
        }
    }

    fn skip_inline_whitespace(&mut self) -> Result<()> {
        while let Some(' ' | '\t') = self.peek()? {
            self.advance()?;
        }
        Ok(())
    }

    fn rest_of_line(&mut self) -> Result<()> {
        loop {
            match self.peek()? {
                Some('\n' | '\r') | None => break,
                Some(_) => {
                    self.advance()?;
                }
            }
        }
        Ok(())
    }

    fn eat_break(&mut self) -> Result<bool> {
        match self.peek()? {
            Some('\r') => {
                self.advance()?;
                if self.peek()? == Some('\n') {
                    self.advance()?;
                }
                Ok(true)
            }
            Some('\n') => {
                self.advance()?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn offset(&self) -> usize {
        self.read.offset()
    }

    fn set_offset(&mut self, offset: usize) {
        self.read.set_offset(offset);
    }

    // ── Error helpers ────────────────────────────────────────────

    fn error(&self, msg: &str) -> Error {
        let (line, col) = line_col(self.read.input(), self.read.offset());
        Error::Message(format!("{line}:{col}: {msg}"))
    }

    fn error_at(&self, msg: &str, offset: usize) -> Error {
        let (line, col) = line_col(self.read.input(), offset);
        Error::Message(format!("{line}:{col}: {msg}"))
    }

    fn end(&mut self) -> Result<()> {
        self.skip_whitespace_and_comments()?;
        if self.is_eof()? {
            Ok(())
        } else {
            Err(self.error("trailing characters after value"))
        }
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<()> {
        loop {
            match self.peek()? {
                Some(' ' | '\t' | '\n' | '\r') => {
                    self.advance()?;
                }
                Some('#') => {
                    self.rest_of_line()?;
                    self.eat_break()?;
                }
                _ => break,
            }
        }
        Ok(())
    }

    // ── Scalar scanning ──────────────────────────────────────────

    /// Scan a double-quoted string, returning the decoded content.
    fn scan_double_quoted(&mut self) -> Result<String> {
        let start = self.offset();
        self.advance()?; // opening `"`
        let mut value = String::new();

        loop {
            match self.peek()? {
                Some('"') => {
                    self.advance()?;
                    return Ok(value);
                }
                Some('\\') => {
                    self.advance()?;
                    let ch = self.parse_escape()?;
                    value.push(ch);
                }
                Some(ch) if ch == '\n' || ch == '\r' => {
                    return Err(self.error_at("expected closing `\"` before line break", start));
                }
                Some(ch) => {
                    if !is_printable(ch) {
                        return Err(
                            self.error(&format!("non-printable character U+{:04X}", ch as u32))
                        );
                    }
                    self.advance()?;
                    value.push(ch);
                }
                None => {
                    return Err(self.error_at("unexpected end of input in string", start));
                }
            }
        }
    }

    /// Parse a double-quoted escape sequence (after consuming the `\`).
    fn parse_escape(&mut self) -> Result<char> {
        let esc_start = self.offset().saturating_sub(1);
        match self.advance()? {
            Some('0') => Ok('\0'),
            Some('a') => Ok('\x07'),
            Some('b') => Ok('\x08'),
            Some('t') => Ok('\t'),
            Some('n') => Ok('\n'),
            Some('v') => Ok('\x0B'),
            Some('f') => Ok('\x0C'),
            Some('r') => Ok('\r'),
            Some('e') => Ok('\x1B'),
            Some(' ') => Ok(' '),
            Some('"') => Ok('"'),
            Some('/') => Ok('/'),
            Some('\\') => Ok('\\'),
            Some('x') => self.parse_hex_escape(2),
            Some('u') => self.parse_hex_escape(4),
            Some('U') => self.parse_hex_escape(8),
            Some(ch) => Err(self.error_at(&format!("invalid escape: \\{ch}"), esc_start)),
            None => Err(self.error_at("unexpected end of input in escape", esc_start)),
        }
    }

    fn parse_hex_escape(&mut self, digits: usize) -> Result<char> {
        let start = self.offset();
        let mut value: u32 = 0;
        for _ in 0..digits {
            match self.advance()? {
                Some(ch) if ch.is_ascii_hexdigit() => {
                    value = value * 16 + ch.to_digit(16).unwrap();
                }
                _ => {
                    return Err(self.error_at(&format!("expected {digits} hex digits"), start));
                }
            }
        }
        char::from_u32(value).ok_or_else(|| {
            self.error_at(&format!("invalid unicode code point U+{value:04X}"), start)
        })
    }

    /// Scan a bare (unquoted) scalar string in the given context.
    /// Returns the raw text without type resolution.
    fn scan_bare_string(&mut self, ctx: Context) -> Result<String> {
        let start = self.offset();

        // ns-plain-first-char
        match self.peek()? {
            Some(ch) if is_plain_first(ch) => {
                self.advance()?;
            }
            Some(ch) => {
                return Err(self.error(&format!("unexpected character `{ch}`")));
            }
            None => {
                return Err(self.error("unexpected end of input"));
            }
        }

        loop {
            let ws_start = self.offset();
            self.skip_inline_whitespace()?;
            let ws_end = self.offset();

            if self.is_break_or_eof()? {
                return Ok(self.read.input()[start..ws_start].to_string());
            }

            match self.peek()? {
                Some('#') if ws_end > ws_start => {
                    self.set_offset(ws_start);
                    return Ok(self.read.input()[start..ws_start].to_string());
                }
                Some('#') => {
                    self.advance()?;
                }
                Some(':') => {
                    let next = self.peek_nth(1)?;
                    if next.is_none()
                        || next == Some(' ')
                        || next == Some('\t')
                        || next == Some('\n')
                        || next == Some('\r')
                    {
                        if ws_end > ws_start {
                            self.set_offset(ws_start);
                        }
                        return Ok(self.read.input()[start..ws_start].to_string());
                    }
                    self.advance()?;
                }
                Some(',' | ']' | '}') if ctx == Context::Flow => {
                    return Ok(self.read.input()[start..ws_start].to_string());
                }
                Some(ch) if !is_printable(ch) => {
                    return Err(self.error(&format!("non-printable character U+{:04X}", ch as u32)));
                }
                Some(_) => {
                    self.advance()?;
                }
                None => {
                    return Ok(self.read.input()[start..ws_start].to_string());
                }
            }
        }
    }

    /// Scan the next scalar value as a string (quoted or bare).
    fn scan_scalar_string(&mut self, ctx: Context) -> Result<String> {
        match self.peek()? {
            Some('"') => self.scan_double_quoted(),
            _ => self.scan_bare_string(ctx),
        }
    }

    /// Parse a bare string as a bool, or return an error.
    fn parse_bool(&mut self) -> Result<bool> {
        let text = self.scan_bare_string(Context::Block)?;
        match text.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(self.error(&format!("expected boolean, got `{text}`"))),
        }
    }

    /// Parse a bare string as an integer in the requested type.
    fn parse_int<T: TryFrom<i64>>(&mut self) -> Result<T>
    where
        T::Error: std::fmt::Display,
    {
        let start = self.offset();
        let text = self.scan_bare_string(Context::Block)?;
        let i = try_parse_int(&text).map_err(|()| self.error_at("integer overflow", start))?;
        let i = i.ok_or_else(|| self.error(&format!("expected integer, got `{text}`")))?;
        T::try_from(i).map_err(|e| self.error(&format!("integer out of range: {e}")))
    }

    /// Parse a bare string as a float in the requested type.
    fn parse_float<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        let text = self.scan_bare_string(Context::Block)?;
        if let Some(f) = try_parse_float(&text) {
            // Also accept integer-shaped text as float
            visitor.visit_f64(f)
        } else if let Ok(Some(i)) = try_parse_int(&text) {
            visitor.visit_f64(i as f64)
        } else {
            Err(self.error(&format!("expected float, got `{text}`")))
        }
    }
}

impl<'de, R: Read<'de>> de::Deserializer<'de> for &mut Deserializer<R> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        match self.peek()? {
            Some('"') => {
                let s = self.scan_double_quoted()?;
                visitor.visit_string(s)
            }
            Some('[') => {
                todo!("deserialize_any: sequence")
            }
            Some('{') => {
                todo!("deserialize_any: mapping")
            }
            Some(_) => {
                // Bare scalar — resolve type
                let start = self.offset();
                let text = self.scan_bare_string(Context::Block)?;
                match text.as_str() {
                    "null" => visitor.visit_unit(),
                    "true" => visitor.visit_bool(true),
                    "false" => visitor.visit_bool(false),
                    _ => match try_parse_int(&text) {
                        Ok(Some(i)) => visitor.visit_i64(i),
                        Err(()) => Err(self.error_at("integer overflow", start)),
                        Ok(None) => {
                            if let Some(f) = try_parse_float(&text) {
                                visitor.visit_f64(f)
                            } else {
                                visitor.visit_string(text)
                            }
                        }
                    },
                }
            }
            None => Err(self.error("unexpected end of input")),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        visitor.visit_i8(self.parse_int()?)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        visitor.visit_i16(self.parse_int()?)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        visitor.visit_i32(self.parse_int()?)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        visitor.visit_i64(self.parse_int()?)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        visitor.visit_u8(self.parse_int()?)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        visitor.visit_u16(self.parse_int()?)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        visitor.visit_u32(self.parse_int()?)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        // i64 can't represent all u64 values, so parse directly
        let start = self.offset();
        let text = self.scan_bare_string(Context::Block)?;
        let val: u64 = parse_unsigned(&text).ok_or_else(|| {
            self.error_at(&format!("expected unsigned integer, got `{text}`"), start)
        })?;
        visitor.visit_u64(val)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        self.parse_float(visitor)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        self.parse_float(visitor)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        let s = self.scan_scalar_string(Context::Block)?;
        let mut chars = s.chars();
        let ch = chars
            .next()
            .ok_or_else(|| self.error("expected a character, got empty string"))?;
        if chars.next().is_some() {
            return Err(self.error("expected a single character"));
        }
        visitor.visit_char(ch)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        let s = self.scan_scalar_string(Context::Block)?;
        visitor.visit_string(s)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        let s = self.scan_scalar_string(Context::Block)?;
        visitor.visit_bytes(s.as_bytes())
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        // Peek at whether this is `null`
        let off = self.offset();
        self.read.fill_to(off + 5)?;
        let is_null = {
            let input = self.read.input();
            if input[off..].starts_with("null") {
                let after = &input.as_bytes()[off + 4..];
                after.is_empty()
                    || after[0] == b' '
                    || after[0] == b'\t'
                    || after[0] == b'\n'
                    || after[0] == b'\r'
                    || after[0] == b'#'
            } else {
                false
            }
        };
        if is_null {
            self.set_offset(off + 4);
            return visitor.visit_none();
        }
        visitor.visit_some(self)
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace_and_comments()?;
        let text = self.scan_bare_string(Context::Block)?;
        if text == "null" {
            visitor.visit_unit()
        } else {
            Err(self.error(&format!("expected null, got `{text}`")))
        }
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        todo!()
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, _visitor: V) -> Result<V::Value> {
        todo!()
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value> {
        todo!()
    }

    fn deserialize_map<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        todo!()
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value> {
        todo!()
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value> {
        todo!()
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_any(visitor)
    }
}

// ── Scalar parsing helpers ───────────────────────────────────────
//
// These duplicate logic from ayml-core's grammar.rs. They operate on
// already-scanned bare text and are the building blocks the deserializer
// uses when serde requests a specific type.

fn is_indicator(ch: char) -> bool {
    matches!(
        ch,
        '-' | ':' | ',' | '[' | ']' | '{' | '}' | '#' | '"' | '\\'
    )
}

fn is_plain_first(ch: char) -> bool {
    if is_indicator(ch) {
        ch == '-' || ch == ':'
    } else {
        !ch.is_ascii_whitespace() && is_printable(ch)
    }
}

fn is_printable(ch: char) -> bool {
    let cp = ch as u32;
    matches!(
        cp,
        0x09 | 0x0A | 0x0D |
        0x20..=0x7E |
        0x85 |
        0xA0..=0xD7FF |
        0xE000..=0xFFFD |
        0x10000..=0x10_FFFF
    )
}

/// Try to parse as an AYML integer. Returns `Ok(Some(i64))` on success,
/// `Ok(None)` if not an integer, or `Err(())` on overflow.
fn try_parse_int(s: &str) -> std::result::Result<Option<i64>, ()> {
    let (unsigned, negative) = match s.strip_prefix('-') {
        Some(rest) => (rest, true),
        None => (s.strip_prefix('+').unwrap_or(s), false),
    };

    let abs = if let Some(bin) = unsigned.strip_prefix("0b") {
        match u64::from_str_radix(bin, 2) {
            Ok(v) => v,
            Err(_) if bin.chars().all(|c| c == '0' || c == '1') => return Err(()),
            Err(_) => return Ok(None),
        }
    } else if let Some(oct) = unsigned.strip_prefix("0o") {
        match u64::from_str_radix(oct, 8) {
            Ok(v) => v,
            Err(_) if oct.chars().all(|c| c.is_ascii_digit() && c < '8') => return Err(()),
            Err(_) => return Ok(None),
        }
    } else if let Some(hex) = unsigned.strip_prefix("0x") {
        match u64::from_str_radix(hex, 16) {
            Ok(v) => v,
            Err(_) if hex.chars().all(|c| c.is_ascii_hexdigit()) => return Err(()),
            Err(_) => return Ok(None),
        }
    } else {
        if unsigned.is_empty() || !unsigned.chars().all(|c| c.is_ascii_digit()) {
            return Ok(None);
        }
        match unsigned.parse::<u64>() {
            Ok(v) => v,
            Err(_) => return Err(()),
        }
    };

    let signed = if negative {
        -i128::from(abs)
    } else {
        i128::from(abs)
    };
    i64::try_from(signed).map(Some).map_err(|_| ())
}

/// Parse an unsigned integer (for u64 values that exceed i64 range).
fn parse_unsigned(s: &str) -> Option<u64> {
    let s = s.strip_prefix('+').unwrap_or(s);

    if let Some(bin) = s.strip_prefix("0b") {
        u64::from_str_radix(bin, 2).ok()
    } else if let Some(oct) = s.strip_prefix("0o") {
        u64::from_str_radix(oct, 8).ok()
    } else if let Some(hex) = s.strip_prefix("0x") {
        u64::from_str_radix(hex, 16).ok()
    } else {
        s.parse().ok()
    }
}

/// Try to parse as an AYML float.
fn try_parse_float(s: &str) -> Option<f64> {
    match s {
        "inf" | "+inf" => return Some(f64::INFINITY),
        "-inf" => return Some(f64::NEG_INFINITY),
        "nan" => return Some(f64::NAN),
        _ => {}
    }

    let (s, negative) = match s.strip_prefix('-') {
        Some(rest) => (rest, true),
        None => (s.strip_prefix('+').unwrap_or(s), false),
    };

    let has_dot = s.contains('.');
    let has_exp = s.contains('e') || s.contains('E');

    if !has_dot && !has_exp {
        return None;
    }

    if has_dot {
        let (int_part, frac_and_exp) = s.split_once('.')?;
        if int_part.is_empty() || !int_part.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        if let Some(e_pos) = frac_and_exp.find(['e', 'E']) {
            let frac = &frac_and_exp[..e_pos];
            let exp = &frac_and_exp[e_pos + 1..];
            if frac.is_empty() || !frac.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            if !valid_exponent(exp) {
                return None;
            }
        } else if frac_and_exp.is_empty() || !frac_and_exp.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
    } else {
        let e_pos = s.find(['e', 'E'])?;
        let int_part = &s[..e_pos];
        let exp = &s[e_pos + 1..];
        if int_part.is_empty() || !int_part.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        if !valid_exponent(exp) {
            return None;
        }
    }

    let full = if negative {
        format!("-{s}")
    } else {
        s.to_string()
    };
    full.parse::<f64>().ok()
}

fn valid_exponent(exp: &str) -> bool {
    let exp = exp
        .strip_prefix('+')
        .or_else(|| exp.strip_prefix('-'))
        .unwrap_or(exp);
    !exp.is_empty() && exp.chars().all(|c| c.is_ascii_digit())
}

fn line_col(source: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(source.len());
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null() {
        let result: () = from_str("null").unwrap();
        assert_eq!(result, ());
    }

    #[test]
    fn test_bool() {
        assert_eq!(from_str::<bool>("true").unwrap(), true);
        assert_eq!(from_str::<bool>("false").unwrap(), false);
    }

    #[test]
    fn test_integers() {
        assert_eq!(from_str::<i32>("42").unwrap(), 42);
        assert_eq!(from_str::<i32>("-17").unwrap(), -17);
        assert_eq!(from_str::<i64>("0").unwrap(), 0);
        assert_eq!(from_str::<u8>("255").unwrap(), 255);
        assert_eq!(from_str::<u32>("0xFF").unwrap(), 255);
        assert_eq!(from_str::<i32>("0b1010").unwrap(), 10);
        assert_eq!(from_str::<i32>("0o77").unwrap(), 63);
    }

    #[test]
    fn test_u64_large() {
        let val: u64 = from_str("18446744073709551615").unwrap();
        assert_eq!(val, u64::MAX);
    }

    #[test]
    fn test_floats() {
        assert_eq!(from_str::<f64>("3.14").unwrap(), 3.14);
        assert_eq!(from_str::<f64>("-0.5").unwrap(), -0.5);
        assert_eq!(from_str::<f64>("1e10").unwrap(), 1e10);
        assert!(from_str::<f64>("inf").unwrap().is_infinite());
        assert!(from_str::<f64>("nan").unwrap().is_nan());
    }

    #[test]
    fn test_string_bare() {
        assert_eq!(from_str::<String>("hello").unwrap(), "hello");
        assert_eq!(from_str::<String>("hello world").unwrap(), "hello world");
    }

    #[test]
    fn test_string_quoted() {
        assert_eq!(from_str::<String>(r#""hello""#).unwrap(), "hello");
        assert_eq!(
            from_str::<String>(r#""hello\nworld""#).unwrap(),
            "hello\nworld"
        );
    }

    #[test]
    fn test_char() {
        assert_eq!(from_str::<char>("a").unwrap(), 'a');
        assert_eq!(from_str::<char>(r#""x""#).unwrap(), 'x');
    }

    #[test]
    fn test_option_none() {
        assert_eq!(from_str::<Option<i32>>("null").unwrap(), None);
    }

    #[test]
    fn test_option_some() {
        assert_eq!(from_str::<Option<i32>>("42").unwrap(), Some(42));
        assert_eq!(
            from_str::<Option<String>>(r#""hello""#).unwrap(),
            Some("hello".to_string())
        );
    }

    #[test]
    fn test_newtype_struct() {
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct Meters(f64);

        assert_eq!(from_str::<Meters>("3.5").unwrap(), Meters(3.5));
    }

    #[test]
    fn test_with_comments() {
        assert_eq!(from_str::<i32>("# a comment\n42").unwrap(), 42);
        assert_eq!(from_str::<i32>("42 # trailing").unwrap(), 42);
    }

    #[test]
    fn test_trailing_whitespace() {
        assert_eq!(from_str::<i32>("42  \n").unwrap(), 42);
    }

    #[test]
    fn test_any_resolution() {
        // Verify deserialize_any resolves types correctly
        #[derive(serde::Deserialize, Debug, PartialEq)]
        #[serde(untagged)]
        enum AnyScalar {
            Bool(bool),
            Int(i64),
            Float(f64),
            Str(String),
        }

        assert_eq!(
            from_str::<AnyScalar>("true").unwrap(),
            AnyScalar::Bool(true)
        );
        assert_eq!(from_str::<AnyScalar>("42").unwrap(), AnyScalar::Int(42));
        assert_eq!(
            from_str::<AnyScalar>("3.14").unwrap(),
            AnyScalar::Float(3.14)
        );
        assert_eq!(
            from_str::<AnyScalar>("hello").unwrap(),
            AnyScalar::Str("hello".to_string())
        );
    }

    #[test]
    fn test_error_trailing() {
        assert!(from_str::<i32>("42 extra").is_err());
    }

    #[test]
    fn test_error_type_mismatch() {
        assert!(from_str::<bool>("42").is_err());
        assert!(from_str::<i32>("hello").is_err());
    }

    #[test]
    fn test_from_slice() {
        assert_eq!(from_slice::<i32>(b"42").unwrap(), 42);
        assert_eq!(from_slice::<String>(b"hello").unwrap(), "hello");
        assert_eq!(from_slice::<String>(b"\"quoted\"").unwrap(), "quoted");
    }

    #[test]
    fn test_from_slice_invalid_utf8() {
        assert!(from_slice::<String>(&[0xFF, 0xFE]).is_err());
    }

    #[test]
    fn test_from_reader() {
        let data = b"42" as &[u8];
        assert_eq!(from_reader::<_, i32>(data).unwrap(), 42);

        let data = b"\"hello\"" as &[u8];
        assert_eq!(from_reader::<_, String>(data).unwrap(), "hello");
    }
}
