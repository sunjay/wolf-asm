use wolf_asm::asm::{
    InstrKind,
    layout::{Opcode, Layout, BitPattern, Reg},
};
use thiserror::Error;

pub type Immediate = i128;
pub type Offset = i16;

#[derive(Debug, Error, Clone)]
pub enum DecodeError {
    #[error("Invalid instruction: opcode `{0}` is not supported")]
    InvalidOpcode(u16),
    #[error("Invalid instruction: unsupported layout")]
    UnsupportedInstructionLayout,
}

/// Represents an argument for an instruction that may be used as a source operand
#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    Register(Reg),
    Immediate(Immediate),
}

/// Represents an argument for an instruction that may be used as a location (address) operand
#[derive(Debug, Clone, PartialEq)]
pub enum Location {
    Register(Reg, Option<Offset>),
    Immediate(Immediate),
}

/// Represents an argument for an instruction that may be used as a destination operand
#[derive(Debug, Clone, PartialEq)]
pub enum Destination {
    Register(Reg),
}

macro_rules! instr {
    (
        $(#[$m:meta])*
        $v:vis enum $instr_enum:ident {
            $(
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
            pub fn decode(instr: u64) -> Result<Self, DecodeError> {
                let opcode = Opcode::read(instr, 0);
                let (kind, opcode_offset) = InstrKind::from_opcode(opcode);
                let args = Layout::from_binary(instr, opcode_offset)
                    .ok_or_else(|| DecodeError::InvalidOpcode(opcode.into_value()))?;

                match kind {
                    $(InstrKind::$instr_variant => Ok($instr_enum::$instr_variant(
                        $instr_struct::args_from_layout(args)?
                    ))),*
                }
            }

            /// Returns the size in bytes that this will have in the generated executable
            pub fn size_bytes(&self) -> u64 {
                // All instructions are currently 8 bytes
                8
            }
        }

        $(
            #[derive(Debug, Clone, PartialEq)]
            $v struct $instr_struct {
                $(pub $instr_field : $instr_value_ty,)*
            }

            impl $instr_struct {
                pub fn args_from_layout(layout: Layout) -> Result<Self, DecodeError> {
                    todo!()
                }
            }
        )*
    };
}

instr! {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Instr {
        Nop(struct Nop {}),

        Add(struct Add {dest: Destination, source: Source}),
        Sub(struct Sub {dest: Destination, source: Source}),

        Mul(struct Mul {dest: Destination, source: Source}),
        Mull(struct Mull {dest_hi: Destination, dest: Destination, source: Source}),
        Mulu(struct Mulu {dest: Destination, source: Source}),
        Mullu(struct Mullu {dest_hi: Destination, dest: Destination, source: Source}),

        Div(struct Div {dest: Destination, source: Source}),
        Divr(struct Divr {dest_rem: Destination, dest: Destination, source: Source}),
        Divu(struct Divu {dest: Destination, source: Source}),
        Divru(struct Divru {dest_rem: Destination, dest: Destination, source: Source}),

        Rem(struct Rem {dest: Destination, source: Source}),
        Remu(struct Remu {dest: Destination, source: Source}),

        And(struct And {dest: Destination, source: Source}),
        Or(struct Or {dest: Destination, source: Source}),
        Xor(struct Xor {dest: Destination, source: Source}),

        Test(struct Test {dest: Source, source: Source}),
        Cmp(struct Cmp {dest: Source, source: Source}),

        Mov(struct Mov {dest: Destination, source: Source}),

        Load1(struct Load1 {dest: Destination, loc: Location}),
        Loadu1(struct Loadu1 {dest: Destination, loc: Location}),
        Load2(struct Load2 {dest: Destination, loc: Location}),
        Loadu2(struct Loadu2 {dest: Destination, loc: Location}),
        Load4(struct Load4 {dest: Destination, loc: Location}),
        Loadu4(struct Loadu4 {dest: Destination, loc: Location}),
        Load8(struct Load8 {dest: Destination, loc: Location}),
        Loadu8(struct Loadu8 {dest: Destination, loc: Location}),

        Store1(struct Store1 {loc: Location, source: Source}),
        Store2(struct Store2 {loc: Location, source: Source}),
        Store4(struct Store4 {loc: Location, source: Source}),
        Store8(struct Store8 {loc: Location, source: Source}),

        Push(struct Push {source: Source}),
        Pop(struct Pop {source: Destination}),

        Jmp(struct Jmp {loc: Location}),
        Je(struct Je {loc: Location}),
        Jne(struct Jne {loc: Location}),
        Jg(struct Jg {loc: Location}),
        Jge(struct Jge {loc: Location}),
        Ja(struct Ja {loc: Location}),
        Jae(struct Jae {loc: Location}),
        Jl(struct Jl {loc: Location}),
        Jle(struct Jle {loc: Location}),
        Jb(struct Jb {loc: Location}),
        Jbe(struct Jbe {loc: Location}),
        Jo(struct Jo {loc: Location}),
        Jno(struct Jno {loc: Location}),
        Jz(struct Jz {loc: Location}),
        Jnz(struct Jnz {loc: Location}),
        Js(struct Js {loc: Location}),
        Jns(struct Jns {loc: Location}),

        Call(struct Call {loc: Location}),
        Ret(struct Ret {}),
    }
}
