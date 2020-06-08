//! An intermediate representation of the program after constants have been substituted and all
//! validations have been completed.

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

macro_rules! instr {
    (
        $(#[$m:meta])*
        $v:vis enum $instr_enum:ident {
            $(
                #[name = $instr_name:literal]
                $instr_variant:ident(struct $instr_struct:ident {
                    $( $instr_field:ident : $instr_value_ty:ident ),* $(,)?
                }),
            )*
        }
    ) => {
        $(#[$m])*
        $v enum $instr_enum {
            $($instr_variant($instr_struct)),*
        }

        impl $instr_enum {
            pub fn validate(instr: ast::Instr, diag: &Diagnostics) -> Self {
                let ast::Instr {name, args} = instr;

                todo!()
            }

            pub fn span(&self) -> Span {
                use $instr_enum::*;
                match self {
                    $($instr_variant(instr) => instr.span),*
                }
            }
        }

        $(
            #[derive(Debug, Clone, PartialEq)]
            $v struct $instr_struct {
                $(pub $instr_field : $instr_value_ty,)*

                /// The span of the entire instruction
                pub span: Span,
            }
        )*
    };
}

instr! {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Instr {
        #[name = "add"]
        Add(struct Add {dest: Destination, source: Source}),
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
    pub fn validate(arg: ast::InstrArg, _diag: &Diagnostics) -> Self {
        match arg {
            ast::InstrArg::Register(reg) => Source::Register(reg),
            ast::InstrArg::Immediate(imm) => Source::Immediate(imm),
            // After const expansion, the only names left are labels
            ast::InstrArg::Name(label) => Source::Label(label),
        }
    }
}

/// Represents an argument for an instruction that may be used as a destination operand
#[derive(Debug, Clone, PartialEq)]
pub enum Destination {
    Register(Register),
}

impl Destination {
    pub fn validate(arg: ast::InstrArg, diag: &Diagnostics) -> Self {
        match arg {
            ast::InstrArg::Register(reg) => Destination::Register(reg),
            _ => {
                let span = arg.span();
                diag.span_error(span, format!("expected a register, found `{}`", arg)).emit();

                // Error Recovery: Just use a default register so the program can keep going
                Destination::Register(Register {
                    kind: RegisterKind::Numbered(0),
                    span,
                })
            },
        }
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
    pub fn validate(arg: ast::InstrArg, _diag: &Diagnostics) -> Self {
        match arg {
            ast::InstrArg::Register(reg) => Location::Register(reg),
            ast::InstrArg::Immediate(imm) => Location::Immediate(imm),
            // After const expansion, the only names left are labels
            ast::InstrArg::Name(label) => Location::Label(label),
        }
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

pub type Register = ast::Register;
pub type RegisterKind = ast::RegisterKind;
/// An immediate value
pub type Immediate = ast::Immediate;
pub type Integer = ast::Integer;
pub type Bytes = ast::Bytes;
pub type Ident = ast::Ident;
