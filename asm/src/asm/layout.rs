use std::marker::PhantomData;

use crate::diagnostics::Diagnostics;
use crate::label_offsets::LabelOffsets;
use crate::asm;

use super::{Source, Destination, Location};

#[derive(Debug, Clone, PartialEq)]
pub struct InstrLayout {
    /// The base opcode for this instruction
    ///
    /// The layout's opcode offset will be added to this value when the binary instruction is
    /// generated to get the final opcode used in the executable.
    pub base_opcode: u16,
    pub layout: Layout,
}

macro_rules! layout {
    (
        $(#[$m:meta])*
        $v:vis enum $layout_enum:ident {
            $(
                #[opcode_offset = $offset:literal]
                $layout_variant:ident(struct $layout_struct:ident (
                    $( $layout_field_ty:ident $(<$field_ty_param:ident>)? ),* $(,)?
                )),
            )*
        }
    ) => {
        $(#[$m])*
        $v enum $layout_enum {
            $(
                $layout_variant($layout_struct)
            ),*
        }

        $(
            #[derive(Debug, Clone, PartialEq)]
            $v struct $layout_struct($($layout_field_ty $(<$field_ty_param>)?),*);

            impl $layout_struct {
                /// Returns the number of bits of the `argument` section of the instruction used by
                /// this particular layout
                pub fn used_arguments_bits() -> u8 {
                    0 $(+ $layout_field_ty $(::<$field_ty_param>)? :: size_bits())*
                }
            }
        )*
    };
}

layout! {
    /// Each supported layout for the `arguments` section of an instruction
    #[derive(Debug, Clone, PartialEq)]
    pub enum Layout {
        #[opcode_offset = 0]
        L1(struct L1(Reg, Reg)),
        #[opcode_offset = 1]
        L2(struct L2(Reg, Imm<S46>)),
        #[opcode_offset = 2]
        L3(struct L3(Reg, Reg, Offset)),
        #[opcode_offset = 3]
        L4(struct L4(Reg, Offset, Imm<S30>)),
        #[opcode_offset = 4]
        L5(struct L5(Imm<S26>, Imm<S26>)),
        #[opcode_offset = 5]
        L6(struct L6(Reg, Reg, Reg)),
        #[opcode_offset = 6]
        L7(struct L7(Reg, Reg, Imm<S40>)),
        #[opcode_offset = 7]
        L8(struct L8(Reg)),
        #[opcode_offset = 8]
        L9(struct L9(Imm<S52>)),
        #[opcode_offset = 9]
        L10(struct L10(Reg, Offset)),
    }
}

pub trait LayoutArguments {
    /// Computes the layout of the `arguments` section of an instruction
    fn layout(self, diag: &Diagnostics, labels: &LabelOffsets) -> Layout;
}

impl LayoutArguments for () {
    fn layout(self, _diag: &Diagnostics, _labels: &LabelOffsets) -> Layout {
        // For zero arguments, layout is unspecified and can be anything
        // No one should rely on this exact representation
        // This was chosen because L1 has an opcode offset of 0 so the opcode is not changed
        Layout::L1(L1(Reg(0), Reg(0)))
    }
}

impl LayoutArguments for (Destination, Source) {
    fn layout(self, diag: &Diagnostics, labels: &LabelOffsets) -> Layout {
        let (dest, src) = self;
        let dest = Dest::new(dest, diag, labels);
        let src = Src::new(src, diag, labels);

        match (dest, src) {
            (Dest::Register(dest_reg), Src::Register(src_reg)) => Layout::L1(L1(
                Reg::new(dest_reg, diag),
                Reg::new(src_reg, diag),
            )),
            (Dest::Register(dest_reg), Src::Immediate(src_imm)) => Layout::L2(L2(
                Reg::new(dest_reg, diag),
                Imm::new(src_imm, diag),
            )),
        }
    }
}

impl LayoutArguments for (Source, Source) {
    fn layout(self, diag: &Diagnostics, labels: &LabelOffsets) -> Layout {
        let (src1, src2) = self;
        let src1 = Src::new(src1, diag, labels);
        let src2 = Src::new(src2, diag, labels);

        match (src1, src2) {
            (Src::Register(src1_reg), Src::Register(src2_reg)) => Layout::L1(L1(
                Reg::new(src1_reg, diag),
                Reg::new(src2_reg, diag),
            )),
            (Src::Register(reg), Src::Immediate(imm)) |
            (Src::Immediate(imm), Src::Register(reg)) => Layout::L2(L2(
                Reg::new(reg, diag),
                Imm::new(imm, diag),
            )),
            (Src::Immediate(src1_imm), Src::Immediate(src2_imm)) => Layout::L5(L5(
                Imm::new(src1_imm, diag),
                Imm::new(src2_imm, diag),
            )),
        }
    }
}

impl LayoutArguments for (Destination, Location) {
    fn layout(self, diag: &Diagnostics, labels: &LabelOffsets) -> Layout {
        let (dest, loc) = self;
        let dest = Dest::new(dest, diag, labels);
        let loc = Loc::new(loc, diag, labels);

        match (dest, loc) {
            (Dest::Register(dest_reg), Loc::Register(loc_reg, None)) => Layout::L1(L1(
                Reg::new(dest_reg, diag),
                Reg::new(loc_reg, diag),
            )),
            (Dest::Register(dest_reg), Loc::Register(loc_reg, Some(offset))) => Layout::L3(L3(
                Reg::new(dest_reg, diag),
                Reg::new(loc_reg, diag),
                Offset::new(offset, diag),
            )),
            (Dest::Register(dest_reg), Loc::Immediate(loc_imm)) => Layout::L2(L2(
                Reg::new(dest_reg, diag),
                Imm::new(loc_imm, diag),
            )),
        }
    }
}

impl LayoutArguments for (Location, Source) {
    fn layout(self, diag: &Diagnostics, labels: &LabelOffsets) -> Layout {
        let (loc, src) = self;
        let loc = Loc::new(loc, diag, labels);
        let src = Src::new(src, diag, labels);
        todo!()
    }
}

impl LayoutArguments for (Destination, Destination, Source) {
    fn layout(self, diag: &Diagnostics, labels: &LabelOffsets) -> Layout {
        let (dest1, dest2, src) = self;
        let dest1 = Dest::new(dest1, diag, labels);
        let dest2 = Dest::new(dest2, diag, labels);
        let src = Src::new(src, diag, labels);
        todo!()
    }
}

impl LayoutArguments for (Source,) {
    fn layout(self, diag: &Diagnostics, labels: &LabelOffsets) -> Layout {
        let (src,) = self;
        let src = Src::new(src, diag, labels);

        match src {
            Src::Register(reg) => Layout::L8(L8(Reg::new(reg, diag))),
            Src::Immediate(imm) => Layout::L9(L9(Imm::new(imm, diag))),
        }
    }
}

impl LayoutArguments for (Destination,) {
    fn layout(self, diag: &Diagnostics, labels: &LabelOffsets) -> Layout {
        let (dest,) = self;
        let dest = Dest::new(dest, diag, labels);

        match dest {
            Dest::Register(reg) => Layout::L8(L8(Reg::new(reg, diag))),
        }
    }
}

impl LayoutArguments for (Location,) {
    fn layout(self, diag: &Diagnostics, labels: &LabelOffsets) -> Layout {
        let (loc,) = self;
        let loc = Loc::new(loc, diag, labels);

        match loc {
            Loc::Register(reg, None) => Layout::L8(L8(Reg::new(reg, diag))),
            Loc::Register(reg, Some(offset)) => Layout::L10(L10(
                Reg::new(reg, diag),
                Offset::new(offset, diag),
            )),
            Loc::Immediate(imm) => Layout::L9(L9(Imm::new(imm, diag))),
        }
    }
}

/// Like `asm::Source`, but with labels resolved to immediates
#[derive(Debug, Clone, PartialEq)]
enum Src {
    Register(asm::Register),
    Immediate(asm::Immediate),
}

impl Src {
    pub fn new(source: asm::Source, diag: &Diagnostics, labels: &LabelOffsets) -> Self {
        match source {
            asm::Source::Register(reg) => Src::Register(reg),
            asm::Source::Immediate(imm) => Src::Immediate(imm),
            asm::Source::Label(label) => Src::Immediate(labels.lookup(&label, diag)),
        }
    }
}

/// Like `asm::Destination`, but with labels resolved to immediates
#[derive(Debug, Clone, PartialEq)]
enum Dest {
    Register(asm::Register),
}

impl Dest {
    pub fn new(source: asm::Destination, _diag: &Diagnostics, _labels: &LabelOffsets) -> Self {
        match source {
            asm::Destination::Register(reg) => Dest::Register(reg),
        }
    }
}

/// Like `Location`, but with labels resolved to immediates
#[derive(Debug, Clone, PartialEq)]
enum Loc {
    Register(asm::Register, Option<asm::Offset>),
    Immediate(asm::Immediate),
}

impl Loc {
    pub fn new(source: asm::Location, diag: &Diagnostics, labels: &LabelOffsets) -> Self {
        match source {
            asm::Location::Register(reg, offset) => Loc::Register(reg, offset),
            asm::Location::Immediate(imm) => Loc::Immediate(imm),
            asm::Location::Label(label) => Loc::Immediate(labels.lookup(&label, diag)),
        }
    }
}

pub trait SizeInBits {
    fn size_bits() -> u8;
}

/// One of the 64 registers, encoded in 6-bits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reg(u8);

impl SizeInBits for Reg {
    fn size_bits() -> u8 {
        6
    }
}

impl Reg {
    pub fn new(reg: asm::Register, _diag: &Diagnostics) -> Self {
        let asm::Register {kind, span: _} = reg;

        match kind {
            asm::RegisterKind::StackPointer => Reg(63),
            asm::RegisterKind::FramePointer => Reg(62),
            // `num` is already guaranteed to be between 0 and 63
            asm::RegisterKind::Numbered(num) => Reg(num),
        }
    }
}

/// An immediate value, encoded with the given size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Imm<S>(i128, PhantomData<S>);

impl<S: SizeInBits> SizeInBits for Imm<S> {
    fn size_bits() -> u8 {
        S::size_bits()
    }
}

impl<S> Imm<S> {
    pub fn new(imm: asm::Immediate, diag: &Diagnostics) -> Self {
        todo!()
    }
}

/// A 16-bit signed offset, encoded in 16-bits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Offset(i16);

impl SizeInBits for Offset {
    fn size_bits() -> u8 {
        16
    }
}

impl Offset {
    pub fn new(offset: asm::Offset, _diag: &Diagnostics) -> Self {
        let asm::Offset {value, span: _} = offset;

        Offset(value)
    }
}

macro_rules! imm_sizes {
    (
        $(
            $(#[$m:meta])*
            $v:vis struct $imm_size_struct:ident($imm_size:literal);
        )*
    ) => {
        $(
            $(#[$m])*
            $v struct $imm_size_struct;

            impl SizeInBits for $imm_size_struct {
                fn size_bits() -> u8 {
                    $imm_size
                }
            }
        )*
    };
}

imm_sizes! {
    /// A 52-bit size
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct S52(52);
    /// A 46-bit size
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct S46(46);
    /// A 40-bit size
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct S40(40);
    /// A 30-bit size
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct S30(30);
    /// A 26-bit size
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct S26(26);
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARGUMENTS_SECTION_SIZE: u8 = 52; // bits

    #[test]
    fn fits_within_arguments_section() {
        assert!(L1::used_arguments_bits() <= ARGUMENTS_SECTION_SIZE);
        assert!(L2::used_arguments_bits() <= ARGUMENTS_SECTION_SIZE);
        assert!(L3::used_arguments_bits() <= ARGUMENTS_SECTION_SIZE);
        assert!(L4::used_arguments_bits() <= ARGUMENTS_SECTION_SIZE);
        assert!(L5::used_arguments_bits() <= ARGUMENTS_SECTION_SIZE);
        assert!(L6::used_arguments_bits() <= ARGUMENTS_SECTION_SIZE);
        assert!(L7::used_arguments_bits() <= ARGUMENTS_SECTION_SIZE);
        assert!(L8::used_arguments_bits() <= ARGUMENTS_SECTION_SIZE);
        assert!(L9::used_arguments_bits() <= ARGUMENTS_SECTION_SIZE);
        assert!(L10::used_arguments_bits() <= ARGUMENTS_SECTION_SIZE);
    }
}
