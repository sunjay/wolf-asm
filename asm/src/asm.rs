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

macro_rules! count_tokens {
    ($t:tt $($ts:tt)*) => {
        1 + count_tokens!($($ts)*)
    };
    () => {
        0
    };
}

macro_rules! instr {
    (
        $(#[$m:meta])*
        $v:vis enum $instr_enum:ident {
            $(
                #[name = $instr_name:literal $(, cond = $cond:expr)?]
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
                #![deny(unreachable_patterns)]
                match &*instr.name.value {
                    $(
                        $instr_name $(if $cond(&instr))? => $instr_enum::$instr_variant(
                            $instr_struct::validate(instr, diag)
                        ),
                    )*

                    _ => {
                        diag.span_error(instr.name.span, format!("unknown instruction `{}`", instr.name.value)).emit();

                        // Error Recovery: Default to a `nop` instruction
                        $instr_enum::Nop(Nop {span: instr.name.span})
                    },
                }
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

            impl $instr_struct {
                pub fn validate(instr: ast::Instr, diag: &Diagnostics) -> Self {
                    let span = instr.span();
                    let ast::Instr {name, mut args} = instr;

                    let expected_args = count_tokens!($($instr_field)*);
                    let provided_args = args.len();

                    // Allows us to access the arguments in the right order using pop() and without
                    // paying to shift the elements every time
                    args.reverse();

                    $(
                        let $instr_field = match args.pop() {
                            Some(arg) => $instr_value_ty::validate(arg, diag),
                            None => {
                                diag.span_error(name.span, format!("expected a {} argument for `{}` instruction (takes {} arguments)", $instr_value_ty::arg_type_name(), name, expected_args)).emit();

                                // Error Recovery: use a default value so we can return *something*
                                // and keep checking for more errors
                                $instr_value_ty::error_default(name.span)
                            },
                        };
                    )*

                    if provided_args > expected_args {
                        diag.span_error(name.span, format!("expected {} arguments for `{}` instruction, found {} arguments", expected_args, name, provided_args)).emit();
                    }

                    Self {
                        $($instr_field,)*
                        span,
                    }
                }
            }
        )*
    };
}

instr! {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Instr {
        #[name = "add"]
        Add(struct Add {dest: Destination, source: Source}),
        #[name = "sub"]
        Sub(struct Sub {dest: Destination, source: Source}),

        #[name = "mul", cond = |instr: &ast::Instr| instr.args.len() == 2]
        Mul(struct Mul {dest: Destination, source: Source}),
        #[name = "mul"]
        MulRem(struct MulRem {dest_hi: Destination, dest: Destination, source: Source}),
        #[name = "mulu", cond = |instr: &ast::Instr| instr.args.len() == 2]
        Mulu(struct Mulu {dest: Destination, source: Source}),
        #[name = "mulu"]
        MuluRem(struct MuluRem {dest_hi: Destination, dest: Destination, source: Source}),

        #[name = "load1"]
        Load1(struct Load1 {dest: Destination, loc: Location}),

        #[name = "nop"]
        Nop(struct Nop {}),
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

    pub fn validate(arg: ast::InstrArg, _diag: &Diagnostics) -> Self {
        match arg {
            ast::InstrArg::Register(reg) => Source::Register(reg),
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
            ast::InstrArg::Register(reg) => Destination::Register(reg),
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

    pub fn validate(arg: ast::InstrArg, _diag: &Diagnostics) -> Self {
        match arg {
            ast::InstrArg::Register(reg) => Location::Register(reg),
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

pub type Register = ast::Register;
pub type RegisterKind = ast::RegisterKind;
/// An immediate value
pub type Immediate = ast::Immediate;
pub type Integer = ast::Integer;
pub type Bytes = ast::Bytes;
pub type Ident = ast::Ident;
