//! Abstract Syntax Tree
//!
//! This is the closest representation to the actual syntax.

use std::fmt;
use std::sync::Arc;
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};

use crate::parser::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Label(Ident),

    Section(Section),

    Include(Include),
    Const(Const),

    StaticData(StaticData),

    Instr(Instr),
}

impl Stmt {
    pub fn is_include(&self) -> bool {
        match self {
            Stmt::Include(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Section {
    pub kind: SectionKind,
    /// The span of the entire section declaration
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    /// The `.static` section
    Static(Span),
    /// The `.code` section
    Code(Span),
}

impl fmt::Display for SectionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use SectionKind::*;
        match self {
            Static(_) => write!(f, ".static"),
            Code(_) => write!(f, ".code"),
        }
    }
}

impl SectionKind {
    pub fn span(self) -> Span {
        use SectionKind::*;
        match self {
            Static(span) |
            Code(span) => span,
        }
    }
}

/// An `.include` directive
#[derive(Debug, Clone, PartialEq)]
pub struct Include {
    pub path: Bytes,
}

/// A `.const` directive
#[derive(Debug, Clone, PartialEq)]
pub struct Const {
    pub name: Ident,
    pub value: Immediate,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StaticData {
    StaticBytes(StaticBytes),
    StaticZero(StaticZero),
    StaticUninit(StaticUninit),
    StaticByteStr(StaticByteStr),
}

/// The `.b1`, `.b2`, `.b4`, or `.b8` static data directive
#[derive(Debug, Clone, PartialEq)]
pub struct StaticBytes {
    /// Either 1, 2, 4, or 8
    pub size: u8,
    pub value: Immediate,
    /// The span of the entire directive
    pub span: Span,
}

/// The `.zero` directive
#[derive(Debug, Clone, PartialEq)]
pub struct StaticZero {
    pub nbytes: Integer,
    /// The span of the entire directive
    pub span: Span,
}

/// The `.uninit` directive
#[derive(Debug, Clone, PartialEq)]
pub struct StaticUninit {
    pub nbytes: Integer,
    /// The span of the entire directive
    pub span: Span,
}

/// The `.bytes` directive
#[derive(Debug, Clone, PartialEq)]
pub struct StaticByteStr {
    pub bytes: Bytes,
    /// The span of the entire directive
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instr {
    /// The name of the instruction (lowercase), e.g. `add`
    pub name: Ident,
    /// The arguments provided to the instruction (possibly empty)
    pub args: Vec<InstrArg>,
}

impl Instr {
    pub fn span(&self) -> Span {
        let Self {name, args} = self;

        match args.last() {
            Some(arg) => name.span.to(arg.span()),
            None => name.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstrArg {
    Register(Register),
    Immediate(Immediate),
    Name(Ident),
}

impl InstrArg {
    pub fn span(&self) -> Span {
        use InstrArg::*;
        match self {
            Register(reg) => reg.span,
            Immediate(imm) => imm.span,
            Name(name) => name.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Register {
    pub kind: RegisterKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegisterKind {
    /// A named register like `$sp` or `$fp`
    Named(Arc<str>),
    /// A numbered register like `$0`, `$1`, `$63`
    Numbered(u8),
}

/// An immediate value
pub type Immediate = Integer;

#[derive(Debug, Clone, PartialEq)]
pub struct Integer {
    /// Value will not exceed the range [i64::min(), u64::max()]
    pub value: i128,
    pub span: Span,
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bytes {
    pub value: Arc<[u8]>,
    pub span: Span,
}

impl fmt::Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = String::from_utf8_lossy(&self.value);
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Ident {
    pub value: Arc<str>,
    pub span: Span,
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl Hash for Ident {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl Borrow<str> for Ident {
    fn borrow(&self) -> &str {
        &self.value
    }
}
