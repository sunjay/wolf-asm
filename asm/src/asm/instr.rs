use crate::ast;
use crate::parser::Span;
use crate::diagnostics::Diagnostics;
use crate::label_offsets::LabelOffsets;

use super::{Source, Destination, Location, layout::InstrLayout};

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

                pub fn layout(self, diag: &Diagnostics, labels: &LabelOffsets) -> InstrLayout {
                    todo!()
                }
            }
        )*
    };
}

instr! {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Instr {
        #[name = "nop"]
        Nop(struct Nop {}),

        #[name = "add"]
        Add(struct Add {dest: Destination, source: Source}),
        #[name = "sub"]
        Sub(struct Sub {dest: Destination, source: Source}),

        #[name = "mul"]
        Mul(struct Mul {dest: Destination, source: Source}),
        #[name = "mull"]
        Mull(struct Mull {dest_hi: Destination, dest: Destination, source: Source}),
        #[name = "mulu"]
        Mulu(struct Mulu {dest: Destination, source: Source}),
        #[name = "mullu"]
        Mullu(struct Mullu {dest_hi: Destination, dest: Destination, source: Source}),

        #[name = "div"]
        Div(struct Div {dest: Destination, source: Source}),
        #[name = "divr"]
        Divr(struct Divr {dest_rem: Destination, dest: Destination, source: Source}),
        #[name = "divu"]
        Divu(struct Divu {dest: Destination, source: Source}),
        #[name = "divru"]
        Divru(struct Divru {dest_rem: Destination, dest: Destination, source: Source}),

        #[name = "rem"]
        Rem(struct Rem {dest: Destination, source: Source}),
        #[name = "remu"]
        Remu(struct Remu {dest: Destination, source: Source}),

        #[name = "and"]
        And(struct And {dest: Destination, source: Source}),
        #[name = "or"]
        Or(struct Or {dest: Destination, source: Source}),
        #[name = "xor"]
        Xor(struct Xor {dest: Destination, source: Source}),

        #[name = "test"]
        Test(struct Test {dest: Source, source: Source}),
        #[name = "cmp"]
        Cmp(struct Cmp {dest: Source, source: Source}),

        #[name = "mov"]
        Mov(struct Mov {dest: Destination, source: Source}),

        #[name = "load1"]
        Load1(struct Load1 {dest: Destination, loc: Location}),
        #[name = "loadu1"]
        Loadu1(struct Loadu1 {dest: Destination, loc: Location}),
        #[name = "load2"]
        Load2(struct Load2 {dest: Destination, loc: Location}),
        #[name = "loadu2"]
        Loadu2(struct Loadu2 {dest: Destination, loc: Location}),
        #[name = "load4"]
        Load4(struct Load4 {dest: Destination, loc: Location}),
        #[name = "loadu4"]
        Loadu4(struct Loadu4 {dest: Destination, loc: Location}),
        #[name = "load8"]
        Load8(struct Load8 {dest: Destination, loc: Location}),
        #[name = "loadu8"]
        Loadu8(struct Loadu8 {dest: Destination, loc: Location}),

        #[name = "store1"]
        Store1(struct Store1 {loc: Location, source: Source}),
        #[name = "store2"]
        Store2(struct Store2 {loc: Location, source: Source}),
        #[name = "store4"]
        Store4(struct Store4 {loc: Location, source: Source}),
        #[name = "store8"]
        Store8(struct Store8 {loc: Location, source: Source}),

        #[name = "push"]
        Push(struct Push {source: Source}),
        #[name = "pop"]
        Pop(struct Pop {source: Destination}),

        #[name = "jmp"]
        Jmp(struct Jmp {loc: Location}),
        #[name = "je"]
        Je(struct Je {loc: Location}),
        #[name = "jne"]
        Jne(struct Jne {loc: Location}),
        #[name = "jg"]
        Jg(struct Jg {loc: Location}),
        #[name = "jge"]
        Jge(struct Jge {loc: Location}),
        #[name = "ja"]
        Ja(struct Ja {loc: Location}),
        #[name = "jae"]
        Jae(struct Jae {loc: Location}),
        #[name = "jl"]
        Jl(struct Jl {loc: Location}),
        #[name = "jle"]
        Jle(struct Jle {loc: Location}),
        #[name = "jb"]
        Jb(struct Jb {loc: Location}),
        #[name = "jbe"]
        Jbe(struct Jbe {loc: Location}),
        #[name = "jo"]
        Jo(struct Jo {loc: Location}),
        #[name = "jno"]
        Jno(struct Jno {loc: Location}),
        #[name = "jz"]
        Jz(struct Jz {loc: Location}),
        #[name = "jnz"]
        Jnz(struct Jnz {loc: Location}),
        #[name = "js"]
        Js(struct Js {loc: Location}),
        #[name = "jns"]
        Jns(struct Jns {loc: Location}),

        #[name = "call"]
        Call(struct Call {loc: Location}),
        #[name = "ret"]
        Ret(struct Ret {}),
    }
}
