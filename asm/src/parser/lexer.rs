use std::sync::Arc;
use std::collections::HashSet;

use crate::diagnostics::Diagnostics;

use super::scanner::Scanner;
use super::token::{self, *};

use TokenKind::*;

pub struct Lexer<'a> {
    scanner: Scanner<'a>,
    diag: &'a Diagnostics,
    interned_strings: HashSet<Arc<str>>,
}

impl<'a> Lexer<'a> {
    pub fn new(scanner: Scanner<'a>, diag: &'a Diagnostics) -> Self {
        Self {
            scanner,
            diag,
            interned_strings: HashSet::new(),
        }
    }

    /// Returns the next token in the input
    pub fn next(&mut self) -> Token {
        self.ignore_whitespace_comments();

        let start = self.scanner.current_pos();
        let current_char = match self.scanner.next() {
            Some(current_char) => current_char,
            None => return self.empty_token(start, Eof),
        };

        // Allows the compiler to help us a bit here
        #[deny(unreachable_patterns)]
        let res = match (current_char, self.scanner.peek()) {
            (b':', _) => Ok(self.byte_token(start, Colon)),
            (b',', _) => Ok(self.byte_token(start, Comma)),
            (b'\n', _) => Ok(self.byte_token(start, Newline)),

            (b'"', _) |
            (b'\'', _) => self.bytes_lit(start, current_char),

            (b'0' ..= b'9', _) |
            (b'-', Some(b'0' ..= b'9')) => self.integer_lit(start, current_char),

            (b'.', Some(b'a' ..= b'z')) |
            (b'.', Some(b'A' ..= b'Z')) |
            (b'.', Some(b'_')) => Ok(self.dot_ident(start)),

            (b'a' ..= b'z', _) |
            (b'A' ..= b'Z', _) |
            (b'_', _) => Ok(self.ident(start)),

            (b'$', _) => self.register(start),

            (ch, _) => {
                let token = self.byte_token(start, Error);
                self.diag.span_error(token.span, format!("unknown start of token `{}`", ch as char)).emit();
                Err(token)
            },
        };

        res.unwrap_or_else(|err| err)
    }

    fn ignore_whitespace_comments(&mut self) {
        while self.ignore_whitespace() || self.ignore_comments() {
            // Keep going until nothing is ignored anymore
        }
    }

    /// Returns true if any whitespace was ignored
    fn ignore_whitespace(&mut self) -> bool {
        let mut ignored = false;
        while let Some(ch) = self.scanner.peek() {
            // A newline doesn't count as whitespace because \n is significant
            if ch.is_ascii_whitespace() && ch != b'\n' {
                self.scanner.next();
                ignored = true;
            } else {
                break;
            }
        }

        ignored
    }

    /// Returns true if any comments were ignored
    fn ignore_comments(&mut self) -> bool {
        let mut ignored = false;

        while let Some(ch) = self.scanner.peek() {
            match ch {
                b'#' | b';' => self.ignore_until_eol(),
                // Keep going until nothing is ignored anymore
                _ => break,
            }
            ignored = true;
        }

        ignored
    }

    /// Ignores until the end of the line
    fn ignore_until_eol(&mut self) {
        // Using peek() because we want to avoid accidentally consuming the newline token
        while let Some(ch) = self.scanner.peek() {
            if ch == b'\n' {
                break;
            }
            self.scanner.next();
        }
    }

    // Parses the remaining byte string literal after `"` or `'`
    fn bytes_lit(&mut self, start: usize, quote: u8) -> Result<Token, Token> {
        let mut unescaped_text = Vec::new();
        loop {
            match self.scanner.next() {
                Some(ch) if ch == quote => break,

                Some(b'\\') => {
                    let unescaped_byte = self.unescape_byte(start)?;
                    unescaped_text.push(unescaped_byte);
                },

                // Unescaped newlines are not allowed
                Some(b'\n') | None => {
                    let token = self.token_to_current(start, Error, None);
                    self.diag.span_error(token.span, "unterminated byte string literal").emit();

                    // Read until the closing quote so we don't get bogus errors
                    while let Some(ch) = self.scanner.next() {
                        if ch == quote {
                            break;
                        }
                    }

                    return Err(token);
                },

                Some(ch) => {
                    unescaped_text.push(ch);
                },
            };
        }

        let value = TokenValue::Bytes(unescaped_text.into());
        Ok(self.token_to_current(start, Literal(LitKind::Bytes), value))
    }

    /// Interprets a byte escape sequence assuming the starting `\` has already been parsed
    fn unescape_byte(&mut self, start: usize) -> Result<u8, Token> {
        match self.scanner.next() {
            Some(b'\\') => Ok(b'\\'),
            Some(b'"') => Ok(b'\"'),
            Some(b'\'') => Ok(b'\''),
            Some(b'n') => Ok(b'\n'),
            Some(b'r') => Ok(b'\r'),
            Some(b't') => Ok(b'\t'),
            Some(b'0') => Ok(b'\0'),

            Some(b'x') => {
                if !matches!(self.scanner.next(), Some(b'{')) {
                    let token = self.byte_token(start, Error);
                    self.diag.span_error(token.span, "invalid hex escape, must look like: `\\x{f}`, `\\x{3A}`").emit();
                    return Err(token);
                }

                let mut digits_buf = String::new();
                let digits = self.digits(true, Some(&mut digits_buf));
                if digits == 0 {
                    let token = self.byte_token(start, Error);
                    self.diag.span_error(token.span, "empty hex escape, must look like: `\\x{f}`, `\\x{3A}`").emit();
                    return Err(token);
                } else if digits > 2 {
                    let token = self.byte_token(start, Error);
                    self.diag.span_error(token.span, "hex byte escape must be 1-2 digits long, e.g. `\\x{f}`, `\\x{3A}`").emit();
                    return Err(token);
                }

                if !matches!(self.scanner.next(), Some(b'}')) {
                    let token = self.byte_token(start, Error);
                    self.diag.span_error(token.span, "invalid hex escape, must look like: `\\x{f}`, `\\x{3A}`").emit();
                    return Err(token);
                }

                // The code above guarantees that we will have a valid number within the range of u8
                let value = u8::from_str_radix(&digits_buf, 16)
                    .expect("bug: should have had a valid u8");

                Ok(value)
            },

            Some(b'b') => {
                if !matches!(self.scanner.next(), Some(b'{')) {
                    let token = self.byte_token(start, Error);
                    self.diag.span_error(token.span, "invalid binary escape, must look like: `\\b{0}`, `\\b{00_01_11_11}`").emit();
                    return Err(token);
                }

                let mut digits_buf = String::new();
                let digits = self.digits(false, Some(&mut digits_buf));
                if digits == 0 {
                    let token = self.byte_token(start, Error);
                    self.diag.span_error(token.span, "empty binary escape, must look like: `\\b{0}`, `\\b{00_01_11_11}`").emit();
                    return Err(token);
                } else if digits > 8 {
                    let token = self.byte_token(start, Error);
                    self.diag.span_error(token.span, "binary byte escape must be 1-8 digits long, e.g. `\\b{0}`, `\\b{00_01_11_11}`").emit();
                    return Err(token);
                }

                if !matches!(self.scanner.next(), Some(b'}')) {
                    let token = self.byte_token(start, Error);
                    self.diag.span_error(token.span, "invalid binary escape, must look like: `\\b{0}`, `\\b{00_01_11_11}`").emit();
                    return Err(token);
                }

                // The code above guarantees that we will have a valid number within the range of u8
                let value = u8::from_str_radix(&digits_buf, 2)
                    .expect("bug: should have had a valid u8");

                Ok(value)
            },

            Some(ch) => {
                let token = self.byte_token(start, Error);
                self.diag.span_error(token.span, format!("unknown character escape: `\\{}`", ch as char)).emit();
                Err(token)
            },
            None => {
                let token = self.token_to_current(start, Error, None);
                self.diag.span_error(token.span, "unterminated byte string literal").emit();
                Err(token)
            },
        }
    }

    /// Parses an integer literal, given a starting digit or negative sign
    ///
    /// The produced value will be 128-bits, but it will not exceed the range [i64::min(), u64::max()]
    fn integer_lit(&mut self, start: usize, start_byte: u8) -> Result<Token, Token> {
        // If the start digit is zero, we may have a hex or binary literal
        let value = match (start_byte, self.scanner.peek()) {
            (b'0', Some(b'x')) => self.hex_lit_value(start)?,
            (b'0', Some(b'b')) => self.binary_lit_value(start)?,
            _ => self.decimal_lit_value(start, start_byte)?,
        };

        if value < i64::MIN as i128 || value > u64::MAX as i128 {
            let token = self.token_to_current(start, Error, None);
            self.diag.span_error(token.span, "integer literal out of 64-bit range").emit();
            return Err(token);
        }

        // An integer cannot be directly followed by an identifier with no whitespace in between
        if matches!(self.scanner.peek(), Some(b'a'..=b'z') | Some(b'A'..=b'Z')) {
            // Skip the first character
            self.scanner.next();
            // Try to avoid bogus errors by skipping the next number or identifier
            let ignore_start = self.scanner.current_pos();
            match self.scanner.next() {
                Some(b'a'..=b'z') | Some(b'A'..=b'Z') => {
                    self.ident(ignore_start);
                },
                Some(current_byte@b'0'..=b'9') => {
                    // Ignore further errors
                    self.integer_lit(ignore_start, current_byte).map(|_| ()).unwrap_or(());
                },
                _ => {},
            }

            let token = self.token_to_current(start, Error, None);
            self.diag.span_error(token.span, "invalid integer literal").emit();
            return Err(token);
        }

        let value = TokenValue::Integer(value);
        Ok(self.token_to_current(start, Literal(LitKind::Integer), value))
    }

    fn hex_lit_value(&mut self, start: usize) -> Result<i128, Token> {
        // Skip `x` character
        self.scanner.next();

        let mut digits_buf = String::new();
        let digits = self.digits(true, Some(&mut digits_buf));
        if digits == 0 {
            let token = self.token_to_current(start, Error, None);
            self.diag.span_error(token.span, "invalid hexadecimal number literal").emit();
            return Err(token);
        }

        match i128::from_str_radix(&digits_buf, 16) {
            Ok(value) => Ok(value),
            Err(_) => {
                let token = self.token_to_current(start, Error, None);
                self.diag.span_error(token.span, "invalid hexadecimal number literal").emit();
                Err(token)
            },
        }
    }

    fn binary_lit_value(&mut self, start: usize) -> Result<i128, Token> {
        // Skip `b` character
        self.scanner.next();

        let mut digits_buf = String::new();
        let digits = self.digits(false, Some(&mut digits_buf));
        if digits == 0 {
            let token = self.token_to_current(start, Error, None);
            self.diag.span_error(token.span, "invalid binary number literal").emit();
            return Err(token);
        }

        match i128::from_str_radix(&digits_buf, 2) {
            Ok(value) => Ok(value),
            Err(_) => {
                let token = self.token_to_current(start, Error, None);
                self.diag.span_error(token.span, "invalid binary number literal").emit();
                Err(token)
            },
        }
    }

    /// Parses a decimal number literal assuming that either a digit or a negative sign has already
    /// been parsed
    fn decimal_lit_value(&mut self, start: usize, start_byte: u8) -> Result<i128, Token> {
        let mut digits_buf = String::new();

        // Add the start digit or negative sign
        digits_buf.push(start_byte as char);

        let digits = self.digits(false, Some(&mut digits_buf));
        if digits == 0 && !start_byte.is_ascii_digit() {
            let token = self.token_to_current(start, Error, None);
            self.diag.span_error(token.span, "invalid decimal number literal").emit();
            return Err(token);
        }

        match i128::from_str_radix(&digits_buf, 10) {
            Ok(value) => Ok(value),
            Err(_) => {
                let token = self.token_to_current(start, Error, None);
                self.diag.span_error(token.span, "invalid decimal number literal").emit();
                Err(token)
            },
        }
    }

    /// Advances the scanner until no more digits are found. Returns the number of digits found.
    ///
    /// The final, non-digit character is NOT consumed
    fn digits(&mut self, hex: bool, mut digit_buf: Option<&mut String>) -> usize {
        let mut digits = 0;
        while let Some(ch) = self.scanner.peek() {
            if ch.is_ascii_digit() || (hex && matches!(ch, b'a' ..= b'f' | b'A' ..= b'F')) {
                if let Some(digit_buf) = &mut digit_buf {
                    digit_buf.push(ch as char);
                }

                digits += 1;
                self.scanner.next();

            } else if ch == b'_' {
                // Skip underscores but don't count them as digits
                self.scanner.next();

            } else {
                break;
            }
        }

        digits
    }

    /// Parses an identifier preceded by a '.', assuming that the '.' character has already been
    /// parsed and that the next character after that is alphabetic
    fn dot_ident(&mut self, start: usize) -> Token {
        // Skip the first letter
        self.scanner.next();

        // Parse the identifier
        self.ident(start);

        let value = self.scanner.slice(start, self.scanner.current_pos());
        // Identifiers are case-insensitive
        let value = value.to_ascii_lowercase();
        let value = TokenValue::Ident(self.intern_str(value));
        self.token_to_current(start, TokenKind::DotIdent, value)
    }

    /// Parses an identifier, assuming that the first character has already been parsed
    ///
    /// Since the first character has already been parsed, this can never fail
    fn ident(&mut self, start: usize) -> Token {
        // We've already got a valid start character, so let's just look for further characters
        while let Some(ch) = self.scanner.peek() {
            if ch.is_ascii_alphanumeric() || ch == b'_' {
                self.scanner.next();
            } else {
                break;
            }
        }

        let value = self.scanner.slice(start, self.scanner.current_pos());
        match token::Keyword::from_str(value) {
            Some(kw) => self.token_to_current(start, Keyword(kw), None),
            None => {
                // Identifiers are case-insensitive
                let value = value.to_ascii_lowercase();
                let value = TokenValue::Ident(self.intern_str(value));
                self.token_to_current(start, TokenKind::Ident, value)
            },
        }
    }

    /// Parses a register, assuming that the starting `$` character has already been parsed
    fn register(&mut self, start: usize) -> Result<Token, Token> {
        let reg_name_start = self.scanner.current_pos();
        match self.scanner.next() {
            Some(b'a' ..= b'z') => {
                let name_token = self.ident(reg_name_start);
                match name_token.kind {
                    TokenKind::Ident => {
                        let name = name_token.unwrap_ident().clone();
                        let value = TokenValue::Register(token::Register::Named(name));
                        Ok(self.token_to_current(start, Register, value))
                    },

                    _ => {
                        let token = self.token_to_current(start, Error, None);
                        self.diag.span_error(token.span, "invalid register name").emit();
                        Err(token)
                    },
                }
            },

            Some(b'0' ..= b'9') => {
                // Try to get any further digits (if any)
                while matches!(self.scanner.peek(), Some(b'0' ..= b'9')) {
                    self.scanner.next();
                }

                let reg_name_end = self.scanner.current_pos();
                let value = match self.scanner.slice(reg_name_start, reg_name_end).parse() {
                    Ok(value) => value,

                    Err(_) => {
                        // The literal is guaranteed by construction to be valid as long as it
                        // isn't too large.
                        let token = self.token_to_current(start, Error, None);
                        self.diag.span_error(token.span, "integer literal is too large").emit();
                        return Err(token);
                    },

                    //TODO: This code is more robust and will work once this feature is stabilized:
                    //  https://github.com/rust-lang/rust/issues/22639
                    //
                    //Err(err) => match err.kind() {
                    //    IntErrorKind::Overflow => {
                    //        let token = self.token_to_current(start, Error, None);
                    //        self.diag.span_error(token.span, "integer literal is too large").emit();
                    //        return token;
                    //    },
                    //
                    //    IntErrorKind::Empty |
                    //    IntErrorKind::InvalidDigit |
                    //    IntErrorKind::Underflow |
                    //    IntErrorKind::Zero => unreachable!("bug: should have been a valid `u8` literal"),
                    //},
                };

                let value = TokenValue::Register(token::Register::Numbered(value));
                Ok(self.token_to_current(start, Register, value))
            },

            Some(_) => {
                let token = self.token_to_current(start, Error, None);
                self.diag.span_error(token.span, "invalid register").emit();
                Err(token)
            },

            None => {
                let token = self.token_to_current(start, Error, None);
                self.diag.span_error(token.span, "unexpected EOF while parsing register").emit();
                Err(token)
            },
        }
    }

    fn empty_token(&self, start: usize, kind: TokenKind) -> Token {
        let span = self.scanner.empty_span(start);
        Token {kind, span, value: None}
    }

    fn byte_token(&self, start: usize, kind: TokenKind) -> Token {
        let span = self.scanner.byte_span(start);
        Token {kind, span, value: None}
    }

    fn token_to_current(&self, start: usize, kind: TokenKind, value: impl Into<Option<TokenValue>>) -> Token {
        let span = self.scanner.span(start, self.scanner.current_pos());
        Token {kind, span, value: value.into()}
    }

    fn intern_str(&mut self, value: String) -> Arc<str> {
        match self.interned_strings.get(&*value) {
            Some(interned) => interned.clone(),
            None => {
                let interned: Arc<str> = value.into();
                self.interned_strings.insert(interned.clone());
                interned
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use parking_lot::RwLock;

    use super::super::{Span, SourceFiles};

    macro_rules! t {
        ($kind:expr) => (
            Token {
                kind: $kind,
                span: Span {start: 0, end: 0},
                value: None,
            }
        );
        ($kind:expr, $value:expr) => (
            Token {
                kind: $kind,
                span: Span {start: 0, end: 0},
                value: Some($value),
            }
        );
    }

    macro_rules! ident {
        ($value:expr) => (
            t!(Ident, TokenValue::Ident($value.into()))
        );
    }

    macro_rules! kw {
        ($kw:ident) => (
            t!(Keyword(Keyword::$kw))
        );
    }

    macro_rules! dot_ident {
        ($value:expr) => (
            t!(DotIdent, TokenValue::Ident($value.into()))
        );
    }

    macro_rules! int {
        ($value:expr) => (
            t!(Literal(LitKind::Integer), TokenValue::Integer($value))
        );
    }

    macro_rules! reg {
        ($value:expr) => (
            t!(Register, TokenValue::Register($value.into()))
        );
    }

    macro_rules! expect_token {
        ($source:literal, $expected:expr) => {
            let source_files = Arc::new(RwLock::new(SourceFiles::default()));
            let root_file = source_files.write().add_source("test.rs", $source);
            let diag = Diagnostics::new(source_files.clone(), termcolor::ColorChoice::Auto);
            let files = source_files.read();
            let scanner = Scanner::new(files.source(root_file));
            let mut lexer = Lexer::new(scanner, &diag);
            let token = lexer.next();
            let expected = $expected;
            assert_eq!(token.kind, expected.kind);
            assert_eq!(token.value, expected.value);
            let token = lexer.next();
            assert_eq!(token.kind, Eof);
        };
    }

    macro_rules! expect_tokens {
        ($source:literal, $expected:expr) => {
            let source_files = Arc::new(RwLock::new(SourceFiles::default()));
            let root_file = source_files.write().add_source("test.rs", $source);
            let diag = Diagnostics::new(source_files.clone(), termcolor::ColorChoice::Auto);
            let files = source_files.read();
            let scanner = Scanner::new(files.source(root_file));
            let mut lexer = Lexer::new(scanner, &diag);
            let expected_tokens: &[Token] = $expected;
            for expected_token in expected_tokens {
                let token = lexer.next();
                assert_eq!(token.kind, expected_token.kind);
                assert_eq!(token.value, expected_token.value);
            }
            // Ensure that the input is exhausted
            let token = lexer.next();
            assert_eq!(token.kind, Eof);
        };
    }

    macro_rules! expect_error {
        ($source:literal) => {
            expect_token!($source, t!(Error));
        };
    }

    #[test]
    fn line_comments() {
        expect_tokens!(b"; 0xInvalidLit", &[]);
        expect_tokens!(b"# 0xInvalid", &[]);
        expect_tokens!(b"###wooooo", &[]);
        expect_tokens!(b";this # is a nested comment ;;; # ##", &[]);
    }

    #[test]
    fn keywords() {
        expect_token!(b"section", kw!(Section));
    }

    #[test]
    fn colon() {
        expect_token!(b":", t!(Colon));
    }

    #[test]
    fn comma() {
        expect_token!(b",", t!(Comma));
    }

    #[test]
    fn newline() {
        expect_token!(b"\n", t!(Newline));
        expect_tokens!(b"\n\n", &[t!(Newline), t!(Newline)]);
        expect_tokens!(b"\n  \n\t \n", &[t!(Newline), t!(Newline), t!(Newline)]);
        expect_token!(b"\t; comment\n", t!(Newline));
        expect_token!(b"# comment\n", t!(Newline));
    }

    #[test]
    fn decimal_literals() {
        expect_token!(b"0", int!(0));
        expect_token!(b"000", int!(0));
        expect_token!(b"013", int!(013));
        expect_token!(b"123", int!(123));
        expect_token!(b"9_999", int!(9999));
        expect_token!(b"-9_999", int!(-9999));
        expect_token!(b"-9223372036854775808", int!(i64::MIN as i128));
        expect_token!(b"18446744073709551615", int!(u64::MAX as i128));
    }

    #[test]
    fn decimal_literals_invalid() {
        // out of range
        expect_error!(b"-9223372036854775809");
        expect_error!(b"18446744073709551616");

        // hex digits
        expect_error!(b"1844A674f4073709C551616");
    }

    #[test]
    fn hex_literals() {
        expect_token!(b"0x0", int!(0x0));
        expect_token!(b"0x000", int!(0x0));
        expect_token!(b"0x013", int!(0x013));
        expect_token!(b"0x123", int!(0x123));
        expect_token!(b"0x9999", int!(0x9999));
        expect_token!(b"0x030AfacbCDdef", int!(0x030AfacbCDdef));
        expect_token!(b"0xffff", int!(0xffff));
        expect_token!(b"0xffff_ffff_ffff_ffff", int!(u64::MAX as i128));
        expect_token!(b"0xFFFF_FFFF_FFFF_FFFF", int!(u64::MAX as i128));
    }

    #[test]
    fn hex_literals_invalid() {
        // out of range
        expect_error!(b"0x1_0000_0000_0000_0000");

        // cannot be negative
        expect_error!(b"-0x0");
        expect_error!(b"-0x1");

        // empty
        expect_error!(b"0x");

        // 'g' is not a letter between 'a' and 'f'
        expect_tokens!(b"0xg", &[t!(Error), ident!("g")]);
    }

    #[test]
    fn binary_literals() {
        expect_token!(b"0b0", int!(0b0));
        expect_token!(b"0b000", int!(0b000));
        expect_token!(b"0b0001", int!(0b0001));
        expect_token!(b"0b1", int!(0b1));
        expect_token!(b"0b11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111", int!(u64::MAX as i128));
    }

    #[test]
    fn binary_literals_invalid() {
        // out of range
        expect_error!(b"0b1_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000");

        // cannot be negative
        expect_error!(b"-0b0");
        expect_error!(b"-0b1");

        // empty
        expect_error!(b"0b");

        // decimal or hex digit
        expect_error!(b"0b2");
        expect_tokens!(b"0bF", &[t!(Error), ident!("f")]);
    }

    #[test]
    fn bytes() {
        let bytes_value = |bytes: &[u8]| TokenValue::Bytes(bytes.into());

        expect_token!(b"\"\"", t!(Literal(LitKind::Bytes), bytes_value(b"")));
        expect_token!(b"''", t!(Literal(LitKind::Bytes), bytes_value(b"")));
        expect_token!(b"\"abc #\\n defok ; okok\"", t!(Literal(LitKind::Bytes), bytes_value(b"abc #\n defok ; okok")));
        expect_token!(b"\'abc #\\n\\t\\r\\'\\\"\\0 defok ; okok\'", t!(Literal(LitKind::Bytes), bytes_value(b"abc #\n\t\r\'\"\0 defok ; okok")));

        expect_token!(
            b"\"\\x{FF} \\b{01} \\b{0} \\x{0} \\x{a} \\b{0001_0001}\"",
            t!(Literal(LitKind::Bytes), bytes_value(b"\xFF \x01 \0 \0 \x0A \x11"))
        );
    }

    #[test]
    fn bytes_invalid_escape() {
        expect_tokens!(b"\"\\x{}\"", &[t!(Error), t!(Error), t!(Error)]);
        expect_tokens!(b"\"\\x{0ff}\"", &[t!(Error), t!(Error), t!(Error)]);
        expect_tokens!(b"\"\\x{100}\"", &[t!(Error), t!(Error), t!(Error)]);

        expect_tokens!(b"\"\\b{}\"", &[t!(Error), t!(Error), t!(Error)]);
        expect_tokens!(b"\"\\b{010001000}\"", &[t!(Error), t!(Error), t!(Error)]);
        expect_tokens!(b"\"\\b{100000000}\"", &[t!(Error), t!(Error), t!(Error)]);
    }

    #[test]
    fn bytes_multiline() {
        expect_error!(b"\"abc\n\"");
        expect_error!(b"'abc\n'");
        expect_error!(b"\"abc
        def'\"");
        expect_error!(b"'abc
        def\"'");
    }

    #[test]
    fn bytes_mismatched_quotes() {
        expect_error!(b"\"'");
        expect_error!(b"'\"");
        expect_error!(b"\"abc '");
        expect_error!(b"'abc \"");
    }

    #[test]
    fn idents() {
        expect_token!(b"a", ident!("a"));
        expect_token!(b"bF", ident!("bf"));
        expect_token!(b"L1", ident!("l1"));
        expect_token!(b"_L1", ident!("_l1"));
        expect_token!(b"abc_efod_fso2190_123___", ident!("abc_efod_fso2190_123___"));
    }

    #[test]
    fn idents_invalid() {
        expect_error!(b"1ab132c");
    }

    #[test]
    fn dot_idents() {
        expect_token!(b".a", dot_ident!(".a"));
        expect_token!(b".bF", dot_ident!(".bf"));
        expect_token!(b".L1", dot_ident!(".l1"));
        expect_token!(b"._L1", dot_ident!("._l1"));
        expect_token!(b".abc_efod_fso2190_123___", dot_ident!(".abc_efod_fso2190_123___"));
    }

    #[test]
    fn dot_idents_invalid() {
        expect_tokens!(b".1ab132c", &[t!(Error), t!(Error)]);
    }

    #[test]
    fn registers() {
        expect_token!(b"$a", reg!("a"));
        expect_token!(b"$sp", reg!("sp"));
        expect_token!(b"$fp", reg!("fp"));
        expect_token!(b"$0", reg!(0));
        expect_token!(b"$1", reg!(1));
        expect_token!(b"$2", reg!(2));
        expect_token!(b"$3", reg!(3));
        expect_token!(b"$4", reg!(4));
        expect_token!(b"$5", reg!(5));
        expect_token!(b"$6", reg!(6));
        expect_token!(b"$7", reg!(7));
        expect_token!(b"$8", reg!(8));
        expect_token!(b"$9", reg!(9));
        expect_token!(b"$10", reg!(10));
        expect_token!(b"$11", reg!(11));
        expect_token!(b"$12", reg!(12));
        expect_token!(b"$13", reg!(13));
        expect_token!(b"$14", reg!(14));
        expect_token!(b"$15", reg!(15));
        expect_token!(b"$16", reg!(16));
        expect_token!(b"$17", reg!(17));
        expect_token!(b"$18", reg!(18));
        expect_token!(b"$19", reg!(19));
        expect_token!(b"$20", reg!(20));
        expect_token!(b"$21", reg!(21));
        expect_token!(b"$22", reg!(22));
        expect_token!(b"$23", reg!(23));
        expect_token!(b"$24", reg!(24));
        expect_token!(b"$25", reg!(25));
        expect_token!(b"$26", reg!(26));
        expect_token!(b"$27", reg!(27));
        expect_token!(b"$28", reg!(28));
        expect_token!(b"$29", reg!(29));
        expect_token!(b"$30", reg!(30));
        expect_token!(b"$31", reg!(31));
        expect_token!(b"$32", reg!(32));
        expect_token!(b"$33", reg!(33));
        expect_token!(b"$34", reg!(34));
        expect_token!(b"$35", reg!(35));
        expect_token!(b"$36", reg!(36));
        expect_token!(b"$37", reg!(37));
        expect_token!(b"$38", reg!(38));
        expect_token!(b"$39", reg!(39));
        expect_token!(b"$40", reg!(40));
        expect_token!(b"$41", reg!(41));
        expect_token!(b"$42", reg!(42));
        expect_token!(b"$43", reg!(43));
        expect_token!(b"$44", reg!(44));
        expect_token!(b"$45", reg!(45));
        expect_token!(b"$46", reg!(46));
        expect_token!(b"$47", reg!(47));
        expect_token!(b"$48", reg!(48));
        expect_token!(b"$49", reg!(49));
        expect_token!(b"$50", reg!(50));
        expect_token!(b"$51", reg!(51));
        expect_token!(b"$52", reg!(52));
        expect_token!(b"$53", reg!(53));
        expect_token!(b"$54", reg!(54));
        expect_token!(b"$55", reg!(55));
        expect_token!(b"$56", reg!(56));
        expect_token!(b"$57", reg!(57));
        expect_token!(b"$58", reg!(58));
        expect_token!(b"$59", reg!(59));
        expect_token!(b"$60", reg!(60));
        expect_token!(b"$61", reg!(61));
        expect_token!(b"$62", reg!(62));
        expect_token!(b"$63", reg!(63));
        expect_token!(b"$128", reg!(128));
    }

    #[test]
    fn registers_invalid() {
        // 256 is greater than u8::MAX
        expect_error!(b"$256");
    }

    #[test]
    fn unknown_token_start() {
        expect_tokens!(b"123\0456", &[int!(123), t!(Error), int!(456)]);
    }
}
