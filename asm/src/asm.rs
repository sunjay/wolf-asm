//! An intermediate representation of the program after constants have been substituted and all
//! validations have been completed.

mod instr;
pub mod layout;

pub use instr::*;

use std::fmt;

use crate::ast;
use crate::parser::Span;
use crate::diagnostics::Diagnostics;

/// The number of registers supported by the machine
pub const REGISTERS: u8 = 64;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// The statements in the `.code` section
    pub code_section: Option<Section>,
    /// The statements in the `.static` section
    pub static_section: Option<Section>,
}

impl Program {
    /// Iterates through all the statements in the program, in order
    pub fn iter_all_stmts(&self) -> impl Iterator<Item = &Stmt> {
        let Program {code_section, static_section} = self;
        code_section.as_ref().map(|section| section.stmts.iter())
            .into_iter()
            .chain(static_section.as_ref().map(|section| section.stmts.iter()))
            .flatten()
    }
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

impl Stmt {
    /// Returns the size in bytes that this will have in the generated executable
    pub fn size_bytes(&self) -> u64 {
        self.kind.size_bytes()
    }
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

    /// Returns the size in bytes that this will have in the generated executable
    pub fn size_bytes(&self) -> u64 {
        use StmtKind::*;
        match self {
            StaticData(data) => data.size_bytes(),
            Instr(instr) => instr.size_bytes(),
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

    /// Returns the size in bytes that this will have in the generated executable
    pub fn size_bytes(&self) -> u64 {
        use StaticData::*;
        match self {
            StaticBytes(data) => data.size_bytes(),
            StaticZero(data) => data.size_bytes(),
            StaticUninit(data) => data.size_bytes(),
            StaticByteStr(data) => data.size_bytes(),
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

impl StaticBytes {
    /// Returns the size in bytes that this will have in the generated executable
    pub fn size_bytes(&self) -> u64 {
        self.value.size_bytes()
    }
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

    /// Returns the size in bytes that this will have in the generated executable
    pub fn size_bytes(&self) -> u64 {
        use StaticBytesValue::*;
        match self {
            B1(_, _) => 1,
            B2(_, _) => 2,
            B4(_, _) => 4,
            B8(_, _) => 8,
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

impl StaticZero {
    /// Returns the size in bytes that this will have in the generated executable
    pub fn size_bytes(&self) -> u64 {
        self.nbytes.value
    }
}

/// The `.uninit` directive
#[derive(Debug, Clone, PartialEq)]
pub struct StaticUninit {
    pub nbytes: Size,
    /// The span of the entire directive
    pub span: Span,
}

impl StaticUninit {
    /// Returns the size in bytes that this will have in the generated executable
    pub fn size_bytes(&self) -> u64 {
        self.nbytes.value
    }
}

/// The `.bytes` directive
#[derive(Debug, Clone, PartialEq)]
pub struct StaticByteStr {
    pub bytes: Bytes,
    /// The span of the entire directive
    pub span: Span,
}

impl StaticByteStr {
    /// Returns the size in bytes that this will have in the generated executable
    pub fn size_bytes(&self) -> u64 {
        self.bytes.value.len() as u64
    }
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
            ast::InstrArg::Register(reg) => {
                let (reg, offset) = Register::validate(reg, diag);
                if let Some(offset) = offset {
                    diag.span_error(offset.span, "source registers do not support offsets").emit();
                }
                Source::Register(reg)
            },
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
            ast::InstrArg::Register(reg) => {
                let (reg, offset) = Register::validate(reg, diag);
                if let Some(offset) = offset {
                    diag.span_error(offset.span, "destination registers do not support offsets").emit();
                }
                Destination::Register(reg)
            },
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
    Register(Register, Option<Offset>),
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
            ast::InstrArg::Register(reg) => {
                let (reg, offset) = Register::validate(reg, diag);
                Location::Register(reg, offset)
            },
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
        }, None)
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
    pub fn validate(reg: ast::Register, diag: &Diagnostics) -> (Self, Option<Offset>) {
        let ast::Register {kind, offset, span} = reg;

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

        let offset = offset.map(|imm| Offset::validate(imm, diag));

        (Self {kind, span}, offset)
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

#[derive(Debug, Clone, PartialEq)]
pub struct Offset {
    pub value: i16,
    pub span: Span,
}

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Offset {
    pub fn validate(imm: ast::Immediate, diag: &Diagnostics) -> Self {
        let ast::Immediate {value, span} = imm;

        let value = if value >= i16::min_value() as i128 && value <= i16::max_value() as i128 {
            value as i16
        } else {
            diag.span_error(span, format!("offset value `{}` must be in the range of a 16-bit signed integer, `{}` to `{}`", value, i16::min_value(), i16::max_value())).emit();

            // Error recovery: return a default value so we can keep producing errors
            0
        };

        Self {value, span}
    }
}

/// An immediate value
pub type Immediate = ast::Immediate;
pub type Integer = ast::Integer;
pub type Bytes = ast::Bytes;
pub type Ident = ast::Ident;
