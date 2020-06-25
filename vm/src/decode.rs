use wolf_asm::asm::{
    InstrKind,
    layout::{
        Opcode,
        Layout,
        BitPattern,
        L1,
        L2,
        L3,
        L4,
        L5,
        L6,
        L7,
        L8,
        L9,
        L10,
        L11,
    },
};
use thiserror::Error;

use crate::machine::Machine;
use crate::execute::{Execute, ExecuteError};
use crate::operands::{Source, Destination, Location};

#[derive(Debug, Error, Clone)]
pub enum DecodeError {
    #[error("Invalid instruction: opcode `{0}` is not supported")]
    InvalidOpcode(u16),
    #[error("Invalid instruction: unsupported layout")]
    UnsupportedInstructionLayout,
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

        impl Execute for $instr_enum {
            fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
                use $instr_enum::*;
                match self {
                    $($instr_variant(instr) => instr.execute(vm)),*
                }
            }
        }

        $(
            #[derive(Debug, Clone, PartialEq)]
            $v struct $instr_struct {
                $(pub $instr_field : $instr_value_ty,)*
            }

            impl $instr_struct {
                pub fn args_from_layout(layout: Layout) -> Result<Self, DecodeError> {
                    let ($($instr_field,)*) = ArgumentsLayout::from_layout(layout)?;
                    Ok(Self {$($instr_field),*})
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
        Not(struct Not {dest: Destination}),

        Test(struct Test {source1: Source, source2: Source}),
        Cmp(struct Cmp {source1: Source, source2: Source}),

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
        Pop(struct Pop {dest: Destination}),

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

macro_rules! match_layout {
    (
        ($layout:expr) {
            $($p:pat => $v:expr),* $(,)?
        }
    ) => (
        match $layout {
            $($p => $v,)*
            _ => return Err(DecodeError::UnsupportedInstructionLayout),
        }
    );
}

pub trait ArgumentsLayout: Sized {
    fn from_layout(layout: Layout) -> Result<Self, DecodeError>;
}

impl ArgumentsLayout for () {
    fn from_layout(_layout: Layout) -> Result<Self, DecodeError> {
        Ok(())
    }
}

impl ArgumentsLayout for (Destination, Source) {
    fn from_layout(layout: Layout) -> Result<Self, DecodeError> {
        match_layout!((layout) {
            Layout::L1(L1(dest, src)) => Ok((dest.into(), src.into())),
            Layout::L2(L2(dest, src)) => Ok((dest.into(), src.into())),
        })
    }
}

impl ArgumentsLayout for (Source, Source) {
    fn from_layout(layout: Layout) -> Result<Self, DecodeError> {
        match_layout!((layout) {
            Layout::L1(L1(src1, src2)) => Ok((src1.into(), src2.into())),
            Layout::L2(L2(src1, src2)) => Ok((src1.into(), src2.into())),
            Layout::L3(L3(src1, src2)) => Ok((src1.into(), src2.into())),
            Layout::L6(L6(src1, src2)) => Ok((src1.into(), src2.into())),
        })
    }
}

impl ArgumentsLayout for (Destination, Location) {
    fn from_layout(layout: Layout) -> Result<Self, DecodeError> {
        match_layout!((layout) {
            Layout::L1(L1(dest, loc)) => Ok((dest.into(), loc.into())),
            Layout::L2(L2(dest, loc)) => Ok((dest.into(), loc.into())),
            Layout::L4(L4(dest, loc, offset)) => Ok((dest.into(), (loc, offset).into())),
        })
    }
}

impl ArgumentsLayout for (Location, Source) {
    fn from_layout(layout: Layout) -> Result<Self, DecodeError> {
        match_layout!((layout) {
            Layout::L1(L1(loc, src)) => Ok((loc.into(), src.into())),
            Layout::L2(L2(loc, src)) => Ok((loc.into(), src.into())),
            Layout::L3(L3(loc, src)) => Ok((loc.into(), src.into())),
            Layout::L4(L4(loc, src, offset)) => Ok(((loc, offset).into(), src.into())),
            Layout::L5(L5(loc, offset, src)) => Ok(((loc, offset).into(), src.into())),
            Layout::L6(L6(loc, src)) => Ok((loc.into(), src.into())),
        })
    }
}

impl ArgumentsLayout for (Destination, Destination, Source) {
    fn from_layout(layout: Layout) -> Result<Self, DecodeError> {
        match_layout!((layout) {
            Layout::L7(L7(dest1, dest2, src)) => Ok((dest1.into(), dest2.into(), src.into())),
            Layout::L8(L8(dest1, dest2, src)) => Ok((dest1.into(), dest2.into(), src.into())),
        })
    }
}

impl ArgumentsLayout for (Source,) {
    fn from_layout(layout: Layout) -> Result<Self, DecodeError> {
        match_layout!((layout) {
            Layout::L9(L9(src)) => Ok((src.into(),)),
            Layout::L10(L10(src)) => Ok((src.into(),)),
        })
    }
}

impl ArgumentsLayout for (Destination,) {
    fn from_layout(layout: Layout) -> Result<Self, DecodeError> {
        match_layout!((layout) {
            Layout::L9(L9(dest)) => Ok((dest.into(),)),
        })
    }
}

impl ArgumentsLayout for (Location,) {
    fn from_layout(layout: Layout) -> Result<Self, DecodeError> {
        match_layout!((layout) {
            Layout::L9(L9(loc)) => Ok((loc.into(),)),
            Layout::L10(L10(loc)) => Ok((loc.into(),)),
            Layout::L11(L11(loc, offset)) => Ok(((loc, offset).into(),)),
        })
    }
}
