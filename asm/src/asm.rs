//! An intermediate representation of the program after constants have been substituted and all
//! validations have been completed.

use std::fmt;
use std::sync::Arc;

use crate::ast;
use crate::parser::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// The statements in the `.code` section
    pub code_section: Option<Section>,
    /// The statements in the `.static` section
    pub static_section: Option<Section>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub section_header_span: Span,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    /// The labels preceding this statement
    ///
    /// The label names are guaranteed to be unique with each other and with any other `Stmt`
    pub labels: Vec<Ident>,
    pub kind: StmtKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    StaticData(StaticData),
    Instr(Instr),
}

impl StmtKind {
    pub fn span(&self) -> Span {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StaticData {
    StaticBytes(StaticBytes),
    StaticZero(StaticZero),
    StaticUninit(StaticUninit),
    StaticByteStr(StaticByteStr),
}

/// The `.b1`, `.b2`, `.b4`, or `.b8` static data directive
///
/// Note that each value is in **little-endian** byte order.
#[derive(Debug, Clone, PartialEq)]
pub enum StaticBytes {
    B1(u8, Span),
    B2([u8; 2], Span),
    B4([u8; 4], Span),
    B8([u8; 8], Span),
}

/// The `.zero` directive
#[derive(Debug, Clone, PartialEq)]
pub struct StaticZero {
    pub nbytes: Size,
}

/// The `.uninit` directive
#[derive(Debug, Clone, PartialEq)]
pub struct StaticUninit {
    pub nbytes: Size,
}

/// The `.bytes` directive
#[derive(Debug, Clone, PartialEq)]
pub struct StaticByteStr {
    pub bytes: Bytes,
}

//TODO: Define validated representation of instructions
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

#[derive(Debug, Clone, PartialEq)]
pub struct Size {
    pub value: u64,
    pub span: Span,
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// An immediate value
pub type Immediate = ast::Immediate;
pub type Integer = ast::Integer;
pub type Bytes = ast::Bytes;
pub type Ident = ast::Ident;
