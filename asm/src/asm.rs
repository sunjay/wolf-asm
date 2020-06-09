//! An intermediate representation of the program after constants have been substituted and all
//! validations have been completed.

mod instr;
pub mod layout;

pub use instr::*;

use std::fmt;

use crate::ast;
use crate::parser::Span;
use crate::diagnostics::Diagnostics;

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
        use StmtKind::*;
        match self {
            StaticData(static_data) => static_data.span(),
            Instr(instr) => instr.span(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StaticData {
    StaticBytes(StaticBytes),
    StaticZero(StaticZero),
    StaticUninit(StaticUninit),
    StaticByteStr(StaticByteStr),
}

impl StaticData {
    pub fn span(&self) -> Span {
        use StaticData::*;
        match self {
            StaticBytes(data) => data.span,
            StaticZero(data) => data.span,
            StaticUninit(data) => data.span,
            StaticByteStr(data) => data.span,
        }
    }
}

/// The `.b1`, `.b2`, `.b4`, or `.b8` static data directive
#[derive(Debug, Clone, PartialEq)]
pub struct StaticBytes {
    pub value: StaticBytesValue,
    /// The span of the entire directive
    pub span: Span,
}

/// Note that each value is in **little-endian** byte order.
#[derive(Debug, Clone, PartialEq)]
pub enum StaticBytesValue {
    B1([u8; 1], Span),
    B2([u8; 2], Span),
    B4([u8; 4], Span),
    B8([u8; 8], Span),
}

impl StaticBytesValue {
    pub fn span(&self) -> Span {
        use StaticBytesValue::*;
        match *self {
            B1(_, span) |
            B2(_, span) |
            B4(_, span) |
            B8(_, span) => span,
        }
    }
}

/// The `.zero` directive
#[derive(Debug, Clone, PartialEq)]
pub struct StaticZero {
    pub nbytes: Size,
    /// The span of the entire directive
    pub span: Span,
}

/// The `.uninit` directive
#[derive(Debug, Clone, PartialEq)]
pub struct StaticUninit {
    pub nbytes: Size,
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

/// Represents an argument for an instruction that may be used as a source operand
#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    Register(Register),
    Immediate(Immediate),
    Label(Ident),
}

impl Source {
    /// Returns a name for this kind of argument that can be used in errors
    pub fn arg_type_name() -> &'static str {
        "source"
    }

    pub fn validate(arg: ast::InstrArg, diag: &Diagnostics) -> Self {
        match arg {
            ast::InstrArg::Register(reg) => Source::Register(Register::validate(reg, diag)),
            ast::InstrArg::Immediate(imm) => Source::Immediate(imm),
            // After const expansion, the only names left are labels
            ast::InstrArg::Name(label) => Source::Label(label),
        }
    }

    /// Returns a default value for this type in case of an error (for error recovery)
    pub fn error_default(span: Span) -> Self {
        Source::Register(Register {
            kind: RegisterKind::Numbered(0),
            span,
        })
    }
}

/// Represents an argument for an instruction that may be used as a destination operand
#[derive(Debug, Clone, PartialEq)]
pub enum Destination {
    Register(Register),
}

impl Destination {
    /// Returns a name for this kind of argument that can be used in errors
    pub fn arg_type_name() -> &'static str {
        "destination"
    }

    pub fn validate(arg: ast::InstrArg, diag: &Diagnostics) -> Self {
        match arg {
            ast::InstrArg::Register(reg) => Destination::Register(Register::validate(reg, diag)),
            _ => {
                let span = arg.span();
                diag.span_error(span, format!("expected a register, found `{}`", arg)).emit();

                // Error Recovery: Just use a default register so the program can keep going
                Self::error_default(span)
            },
        }
    }

    /// Returns a default value for this type in case of an error (for error recovery)
    pub fn error_default(span: Span) -> Self {
        Destination::Register(Register {
            kind: RegisterKind::Numbered(0),
            span,
        })
    }
}

/// Represents an argument for an instruction that may be used as a location (address) operand
#[derive(Debug, Clone, PartialEq)]
pub enum Location {
    Register(Register),
    Immediate(Immediate),
    Label(Ident),
}

impl Location {
    /// Returns a name for this kind of argument that can be used in errors
    pub fn arg_type_name() -> &'static str {
        "location"
    }

    pub fn validate(arg: ast::InstrArg, diag: &Diagnostics) -> Self {
        match arg {
            ast::InstrArg::Register(reg) => Location::Register(Register::validate(reg, diag)),
            ast::InstrArg::Immediate(imm) => Location::Immediate(imm),
            // After const expansion, the only names left are labels
            ast::InstrArg::Name(label) => Location::Label(label),
        }
    }

    /// Returns a default value for this type in case of an error (for error recovery)
    pub fn error_default(span: Span) -> Self {
        Location::Register(Register {
            kind: RegisterKind::Numbered(0),
            span,
        })
    }
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

#[derive(Debug, Clone, PartialEq)]
pub struct Register {
    pub kind: RegisterKind,
    pub span: Span,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "${}", self.kind)
    }
}

impl Register {
    pub fn validate(reg: ast::Register, diag: &Diagnostics) -> Self {
        let ast::Register {kind, span} = reg;

        let kind = match kind {
            ast::RegisterKind::Named(name) if &*name == "sp" => {
                RegisterKind::StackPointer
            },

            ast::RegisterKind::Named(name) if &*name == "fp" => {
                RegisterKind::FramePointer
            },

            ast::RegisterKind::Numbered(num) if num <= 63 => {
                RegisterKind::Numbered(num)
            },

            _ => {
                diag.span_error(span, format!("invalid register `${}`", kind))
                    .span_note(span, "registers must be `$0` to `$63`, `$sp`, or `$fp`").emit();

                // Error recovery: return a default register so we can keep producing errors
                RegisterKind::Numbered(0)
            },
        };

        Self {kind, span}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterKind {
    /// The `$sp` register
    StackPointer,
    /// The `$fp` register
    FramePointer,
    /// A numbered register like `$0`, `$1`, `$63`
    ///
    /// This value is guaranteed to be between 0 and 63 (inclusive)
    Numbered(u8),
}

impl fmt::Display for RegisterKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RegisterKind::*;
        match self {
            StackPointer => write!(f, "sp"),
            FramePointer => write!(f, "fp"),
            Numbered(num) => write!(f, "{}", num),
        }
    }
}

/// An immediate value
pub type Immediate = ast::Immediate;
pub type Integer = ast::Integer;
pub type Bytes = ast::Bytes;
pub type Ident = ast::Ident;
