use crate::error::{Error, ErrorKind, Span};

/// Low-level character scanner over the input string.
///
/// Tracks byte offset, line, and column. Provides helpers for the character
/// productions defined in the spec (c-printable, b-break, s-white, etc.).
pub struct Scanner<'a> {
    pub input: &'a str,
    bytes: &'a [u8],
    /// Current byte offset into the input.
    pub offset: usize,
}

impl<'a> Scanner<'a> {
    pub const fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            offset: 0,
        }
    }

    /// The full source input (for error reporting).
    pub const fn source(&self) -> &'a str {
        self.input
    }

    /// True if the scanner has reached the end of input.
    pub const fn is_eof(&self) -> bool {
        self.offset >= self.input.len()
    }

    /// Peek at the current character without advancing.
    pub fn peek(&self) -> Option<char> {
        self.input[self.offset..].chars().next()
    }

    /// Peek at the next n-th character (0 = current).
    pub fn peek_nth(&self, n: usize) -> Option<char> {
        self.input[self.offset..].chars().nth(n)
    }

    /// Advance past the current character and return it.
    pub fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }

    /// Advance if the current character equals `expected`.
    pub fn eat(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Advance if the upcoming input starts with `s`.
    pub fn eat_str(&mut self, s: &str) -> bool {
        if self.input[self.offset..].starts_with(s) {
            self.offset += s.len();
            true
        } else {
            false
        }
    }

    /// Consume and return the rest of the current line (up to but not
    /// including the line break). Does not consume the line break.
    pub fn rest_of_line(&mut self) -> &'a str {
        let start = self.offset;
        while !self.is_eof() {
            match self.peek() {
                Some('\n' | '\r') | None => break,
                Some(ch) => self.offset += ch.len_utf8(),
            }
        }
        &self.input[start..self.offset]
    }

    /// Consume a line break (CR, LF, or CRLF). Returns true if consumed.
    pub fn eat_break(&mut self) -> bool {
        if self.eat('\r') {
            self.eat('\n'); // CRLF
            true
        } else {
            self.eat('\n')
        }
    }

    /// Count leading spaces at the current position without consuming them.
    pub fn count_spaces(&self) -> usize {
        let mut count = 0;
        let slice = &self.bytes[self.offset..];
        for &b in slice {
            if b == b' ' {
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    /// Consume exactly `n` spaces. Returns false (and does not advance) if
    /// there are fewer than `n` spaces or a tab is encountered.
    pub fn eat_spaces(&mut self, n: usize) -> bool {
        let slice = &self.bytes[self.offset..];
        if slice.len() < n {
            return false;
        }
        for &b in &slice[..n] {
            if b != b' ' {
                return false;
            }
        }
        self.offset += n;
        true
    }

    /// Skip whitespace (spaces and tabs) on the current line.
    pub fn skip_inline_whitespace(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t')) {
            self.advance();
        }
    }

    /// True if the current character is a space or tab.
    pub fn is_white(&self) -> bool {
        matches!(self.peek(), Some(' ' | '\t'))
    }

    /// True if the current character is a line break.
    pub fn is_break(&self) -> bool {
        matches!(self.peek(), Some('\n' | '\r'))
    }

    /// True if the current character is a line break or EOF.
    pub fn is_break_or_eof(&self) -> bool {
        self.is_eof() || self.is_break()
    }

    /// Check if a character is in the c-printable set per the spec.
    pub const fn is_printable(ch: char) -> bool {
        let cp = ch as u32;
        matches!(cp,
            0x09 | 0x0A | 0x0D |
            0x20..=0x7E |
            0x85 |
            0xA0..=0xD7FF |
            0xE000..=0xFFFD |
            0x10000..=0x10_FFFF
        )
    }

    /// Create an error at the current position.
    pub fn error(&self, kind: ErrorKind) -> Error {
        Error::new(kind, Span::point(self.offset), self.input)
    }

    /// Create an error at a specific offset.
    pub fn error_at(&self, kind: ErrorKind, offset: usize) -> Error {
        Error::new(kind, Span::point(offset), self.input)
    }

    /// Parse a double-quoted escape sequence (after consuming the `\`).
    /// Returns the decoded character.
    pub fn parse_escape(&mut self) -> Result<char, Error> {
        let esc_start = self.offset - 1; // offset of the `\`
        match self.advance() {
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
            Some(ch) => Err(Error::new(
                ErrorKind::InvalidEscape(format!("\\{ch}")),
                Span::new(esc_start, self.offset),
                self.input,
            )),
            None => Err(self.error_at(ErrorKind::UnexpectedEof, esc_start)),
        }
    }

    fn parse_hex_escape(&mut self, digits: usize) -> Result<char, Error> {
        let start = self.offset;
        let mut value: u32 = 0;
        for _ in 0..digits {
            match self.advance() {
                Some(ch) if ch.is_ascii_hexdigit() => {
                    value = value * 16 + ch.to_digit(16).unwrap();
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidEscape(format!("expected {digits} hex digits")),
                        Span::new(start, self.offset),
                        self.input,
                    ));
                }
            }
        }
        char::from_u32(value).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidEscape(format!("invalid unicode code point U+{value:04X}")),
                Span::new(start, self.offset),
                self.input,
            )
        })
    }
}
