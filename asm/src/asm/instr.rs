use crate::ast;
use crate::parser::Span;
use crate::diagnostics::Diagnostics;
use crate::label_offsets::LabelOffsets;

use super::{
    Source,
    Destination,
    Location,
    layout::{InstrLayout, LayoutArguments, Opcode},
};

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
        $v:vis enum $instr_enum:ident / $instr_kind_enum:ident {
            $(
                #[opcode = $opcode:literal, name = $instr_name:literal $(, cond = $cond:expr)?]
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

            /// Returns the size in bytes that this will have in the generated executable
            pub fn size_bytes(&self) -> u64 {
                // All instructions are currently 8 bytes
                8
            }

            pub fn layout(self, diag: &Diagnostics, labels: &LabelOffsets) -> InstrLayout {
                use $instr_enum::*;
                match self {
                    $($instr_variant(instr) => instr.layout(diag, labels)),*
                }
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $v enum $instr_kind_enum {
            $($instr_variant),*
        }

        impl $instr_kind_enum {
            /// Returns the instruction kind and opcode offset for the given
            /// opcode
            pub fn from_opcode(opcode: Opcode) -> (Self, u16) {
                let opcode = opcode.into_value();
                let opcodes = [$(($opcode, $instr_kind_enum::$instr_variant)),*];
                // index-1 works because the first opcode is 0 and u16::MIN == 0
                let op_index = opcodes.binary_search_by_key(&opcode, |&(opcode, _)| opcode)
                    .unwrap_or_else(|index| index.wrapping_sub(1));
                let (instr_opcode, kind) = unsafe { *opcodes.get_unchecked(op_index) };
                (kind, opcode - instr_opcode)
            }

            /// Returns the size in bytes that this will have in the generated executable
            pub fn size_bytes(&self) -> usize {
                // All instructions are currently 8 bytes
                8
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
                /// The base opcode of this instruction
                pub const OPCODE: u16 = $opcode;

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

                pub fn layout(self, diag: &Diagnostics, labels: &LabelOffsets) -> InstrLayout {
                    let Self {$($instr_field,)* span: _} = self;

                    InstrLayout {
                        base_opcode: Self::OPCODE,
                        layout: ($($instr_field,)*).layout(diag, labels),
                    }
                }
            }
        )*
    };
}

instr! {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Instr/InstrKind {
        #[opcode = 0, name = "nop"]
        Nop(struct Nop {}),

        #[opcode = 12, name = "add"]
        Add(struct Add {dest: Destination, source: Source}),
        #[opcode = 24, name = "sub"]
        Sub(struct Sub {dest: Destination, source: Source}),

        #[opcode = 36, name = "mul"]
        Mul(struct Mul {dest: Destination, source: Source}),
        #[opcode = 48, name = "mull"]
        Mull(struct Mull {dest_hi: Destination, dest: Destination, source: Source}),
        #[opcode = 60, name = "mulu"]
        Mulu(struct Mulu {dest: Destination, source: Source}),
        #[opcode = 72, name = "mullu"]
        Mullu(struct Mullu {dest_hi: Destination, dest: Destination, source: Source}),

        #[opcode = 84, name = "div"]
        Div(struct Div {dest: Destination, source: Source}),
        #[opcode = 96, name = "divr"]
        Divr(struct Divr {dest_rem: Destination, dest: Destination, source: Source}),
        #[opcode = 108, name = "divu"]
        Divu(struct Divu {dest: Destination, source: Source}),
        #[opcode = 120, name = "divru"]
        Divru(struct Divru {dest_rem: Destination, dest: Destination, source: Source}),

        #[opcode = 132, name = "rem"]
        Rem(struct Rem {dest: Destination, source: Source}),
        #[opcode = 144, name = "remu"]
        Remu(struct Remu {dest: Destination, source: Source}),

        #[opcode = 156, name = "and"]
        And(struct And {dest: Destination, source: Source}),
        #[opcode = 168, name = "or"]
        Or(struct Or {dest: Destination, source: Source}),
        #[opcode = 180, name = "xor"]
        Xor(struct Xor {dest: Destination, source: Source}),
        #[opcode = 192, name = "not"]
        Not(struct Not {dest: Destination}),

        #[opcode = 204, name = "test"]
        Test(struct Test {source1: Source, source2: Source}),
        #[opcode = 216, name = "cmp"]
        Cmp(struct Cmp {source1: Source, source2: Source}),

        #[opcode = 228, name = "mov"]
        Mov(struct Mov {dest: Destination, source: Source}),

        #[opcode = 240, name = "load1"]
        Load1(struct Load1 {dest: Destination, loc: Location}),
        #[opcode = 252, name = "loadu1"]
        Loadu1(struct Loadu1 {dest: Destination, loc: Location}),
        #[opcode = 264, name = "load2"]
        Load2(struct Load2 {dest: Destination, loc: Location}),
        #[opcode = 276, name = "loadu2"]
        Loadu2(struct Loadu2 {dest: Destination, loc: Location}),
        #[opcode = 288, name = "load4"]
        Load4(struct Load4 {dest: Destination, loc: Location}),
        #[opcode = 300, name = "loadu4"]
        Loadu4(struct Loadu4 {dest: Destination, loc: Location}),
        #[opcode = 312, name = "load8"]
        Load8(struct Load8 {dest: Destination, loc: Location}),
        #[opcode = 324, name = "loadu8"]
        Loadu8(struct Loadu8 {dest: Destination, loc: Location}),

        #[opcode = 336, name = "store1"]
        Store1(struct Store1 {loc: Location, source: Source}),
        #[opcode = 348, name = "store2"]
        Store2(struct Store2 {loc: Location, source: Source}),
        #[opcode = 360, name = "store4"]
        Store4(struct Store4 {loc: Location, source: Source}),
        #[opcode = 372, name = "store8"]
        Store8(struct Store8 {loc: Location, source: Source}),

        #[opcode = 384, name = "push"]
        Push(struct Push {source: Source}),
        #[opcode = 396, name = "pop"]
        Pop(struct Pop {dest: Destination}),

        #[opcode = 408, name = "jmp"]
        Jmp(struct Jmp {loc: Location}),
        #[opcode = 420, name = "je"]
        Je(struct Je {loc: Location}),
        #[opcode = 432, name = "jne"]
        Jne(struct Jne {loc: Location}),
        #[opcode = 444, name = "jg"]
        Jg(struct Jg {loc: Location}),
        #[opcode = 456, name = "jge"]
        Jge(struct Jge {loc: Location}),
        #[opcode = 468, name = "ja"]
        Ja(struct Ja {loc: Location}),
        #[opcode = 480, name = "jae"]
        Jae(struct Jae {loc: Location}),
        #[opcode = 492, name = "jl"]
        Jl(struct Jl {loc: Location}),
        #[opcode = 504, name = "jle"]
        Jle(struct Jle {loc: Location}),
        #[opcode = 516, name = "jb"]
        Jb(struct Jb {loc: Location}),
        #[opcode = 528, name = "jbe"]
        Jbe(struct Jbe {loc: Location}),
        #[opcode = 540, name = "jo"]
        Jo(struct Jo {loc: Location}),
        #[opcode = 552, name = "jno"]
        Jno(struct Jno {loc: Location}),
        #[opcode = 564, name = "jz"]
        Jz(struct Jz {loc: Location}),
        #[opcode = 576, name = "jnz"]
        Jnz(struct Jnz {loc: Location}),
        #[opcode = 588, name = "js"]
        Js(struct Js {loc: Location}),
        #[opcode = 600, name = "jns"]
        Jns(struct Jns {loc: Location}),

        #[opcode = 612, name = "call"]
        Call(struct Call {loc: Location}),
        #[opcode = 624, name = "ret"]
        Ret(struct Ret {}),
    }
}
