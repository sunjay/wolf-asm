use std::marker::PhantomData;

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
    /// Each supported instruction layout
    #[derive(Debug, Clone, PartialEq)]
    pub enum Layout {
        #[opcode_offset = 0]
        L1(struct L1(Reg, Reg)),
        #[opcode_offset = 1]
        L2(struct L2(Reg, Imm<S46>)),
        #[opcode_offset = 2]
        L3(struct L3(Reg, Reg, Offset)),
        #[opcode_offset = 3]
        L4(struct L4(Reg, Reg, Imm<S30>)),
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

/// An immediate value, encoded with the given size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Imm<S>(i128, PhantomData<S>);

impl<S: SizeInBits> SizeInBits for Imm<S> {
    fn size_bits() -> u8 {
        S::size_bits()
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
