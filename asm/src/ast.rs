use std::fmt;
use std::sync::Arc;

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

    StaticBytes(StaticBytes),
    StaticZero(StaticZero),
    StaticUninit(StaticUninit),
    StaticByteStr(StaticByteStr),

    Instr(Instr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// The `.static` section
    Static,
    /// The `.code` section
    Code,
}

/// An `.include` directive
#[derive(Debug, Clone, PartialEq)]
pub struct Include {
    pub path: Arc<[u8]>,
}

/// A `.const` directive
#[derive(Debug, Clone, PartialEq)]
pub struct Const {
    pub name: Ident,
    pub value: Immediate,
}

/// The `.b1`, `.b2`, `.b4`, or `.b8` static data directive
#[derive(Debug, Clone, PartialEq)]
pub struct StaticBytes {
    /// Either 1, 2, 4, or 8
    pub size: u8,
    pub value: Immediate,
}

/// The `.zero` directive
#[derive(Debug, Clone, PartialEq)]
pub struct StaticZero {
    pub nbytes: i128,
}

/// The `.uninit` directive
#[derive(Debug, Clone, PartialEq)]
pub struct StaticUninit {
    pub nbytes: i128,
}

/// The `.bytes` directive
#[derive(Debug, Clone, PartialEq)]
pub struct StaticByteStr {
    pub bytes: Arc<[u8]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instr {
    /// The name of the instruction (lowercase), e.g. `add`
    pub name: Ident,
    /// The arguments provided to the instruction (possibly empty)
    pub args: Vec<InstrArg>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstrArg {
    Register(Register),
    Immediate(Immediate),
    Name(Ident),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Register {
    /// A named register like `$sp` or `$fp`
    Named(Arc<str>),
    /// A numbered register like `$0`, `$1`, `$63`
    Numbered(u8),
}

/// An immediate value
pub type Immediate = i128;

#[derive(Debug, Clone, PartialEq)]
pub struct Ident {
    pub value: Arc<str>,
    pub span: Span,
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
