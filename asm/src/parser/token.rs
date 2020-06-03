use std::sync::Arc;

use super::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Section,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterKind {
    /// A named register like `$sp` or `$fp`
    Named,
    /// A numbered register like `$0`, `$1`, `$63`
    Numbered,
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
    Register(RegisterKind),

    /// A literal
    Literal(LitKind),

    /// A `:` character
    Colon,

    /// The `\n` character
    Newline,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Register {
    Named(Arc<str>),
    Numbered(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    /// An interned string representing the identifier
    ///
    /// For `DotIdent` tokens, this is the identifier *after* the '.' character
    Ident(Arc<str>),

    /// The value of a named or numbered register
    Register(Register),

    /// The unescaped bytes from a byte string literal
    Bytes(Arc<[u8]>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    kind: TokenKind,
    span: Span,
    /// Some token kinds have data associated with them
    value: Option<TokenValue>,
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

    /// Returns the value of this token as a byte string or panics
    pub fn unwrap_bytes(&self) -> &Arc<[u8]> {
        match &self.value {
            Some(TokenValue::Bytes(bytes)) => bytes,
            _ => unreachable!("bug: expected a byte string"),
        }
    }
}
