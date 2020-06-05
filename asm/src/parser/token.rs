use std::fmt;
use std::sync::Arc;

use super::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Section,
}

impl Keyword {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "section" => Some(Keyword::Section),
            _ => None,
        }
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Keyword::*;
        match self {
            Section => write!(f, "`section`"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LitKind {
    /// An integer literal, e.g. `0`, `1`, `-402`, `1_000_000`, `0x1f3`, `0b0100_1000`
    Integer,

    /// A string literal, interpreted as a series of bytes
    ///
    /// The literal may contain escaped characters which will be unescaped during lexing.
    ///
    /// e.g. `'abc'`, `'hello, world!\n'`
    Bytes,
}

impl fmt::Display for LitKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use LitKind::*;
        match self {
            Integer => write!(f, "an integer"),
            Bytes => write!(f, "a byte string literal"),
        }
    }
}

/// The different kinds of tokens that can be produced by the lexer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// A keyword
    Keyword(Keyword),
    /// A period character immediately followed by an identifier with no whitespace between them
    DotIdent,
    /// An identifier
    Ident,

    /// A register, e.g. `$0`, `$1`, `$63`, `$sp`, `$fp`, etc.
    Register,

    /// A literal
    Literal(LitKind),

    /// A `:` character
    Colon,
    /// A `,` character
    Comma,

    /// The `\n` character
    Newline,

    /// The end of a file
    Eof,

    /// A placeholder token used to indicate an error but still allow lexing to continue
    Error,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TokenKind::*;
        match self {
            Keyword(kw) => write!(f, "{}", kw),
            DotIdent => write!(f, "`.`"),
            Ident => write!(f, "an identifier"),
            Register => write!(f, "a register"),
            Literal(lit) => write!(f, "{}", lit),
            Colon => write!(f, "`:`"),
            Comma => write!(f, "`,`"),
            Newline => write!(f, "a newline"),
            Eof => write!(f, "end of file"),

            Error => panic!("The Error token kind should not be formatted"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Register {
    /// A named register like `$sp` or `$fp`
    Named(Arc<str>),
    /// A numbered register like `$0`, `$1`, `$63`
    Numbered(u8),
}

impl From<u8> for Register {
    fn from(value: u8) -> Self {
        Register::Numbered(value)
    }
}

impl<'a> From<&'a str> for Register {
    fn from(value: &'a str) -> Self {
        Register::Named(value.into())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    /// An interned string representing the identifier, e.g. `label_name`, `.const`
    Ident(Arc<str>),

    /// The value of a named or numbered register
    Register(Register),

    /// An integer literal value, up to 64-bits in size
    ///
    /// Needs to be 128 bits to fit the range of both i64 and u64
    Integer(i128),

    /// The unescaped bytes from a byte string literal
    Bytes(Arc<[u8]>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    /// Some token kinds have data associated with them
    pub value: Option<TokenValue>,
}

impl Token {
    /// Returns the value of this token as an identifier or panics
    pub fn unwrap_ident(&self) -> &Arc<str> {
        match &self.value {
            Some(TokenValue::Ident(ident)) => ident,
            _ => unreachable!("bug: expected an identifier"),
        }
    }

    /// Returns the value of this token as a register or panics
    pub fn unwrap_register(&self) -> &Register {
        match &self.value {
            Some(TokenValue::Register(reg)) => reg,
            _ => unreachable!("bug: expected a register"),
        }
    }

    /// Returns the value of this token as an integer or panics
    pub fn unwrap_integer(&self) -> i128 {
        match self.value {
            Some(TokenValue::Integer(value)) => value,
            _ => unreachable!("bug: expected a register"),
        }
    }

    /// Returns the value of this token as a byte string or panics
    pub fn unwrap_bytes(&self) -> &Arc<[u8]> {
        match &self.value {
            Some(TokenValue::Bytes(bytes)) => bytes,
            _ => unreachable!("bug: expected a byte string"),
        }
    }
}
