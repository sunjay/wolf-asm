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

/// One of the 64 registers, encoded in 6-bits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reg(u8);

/// An immediate value, encoded with the given size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Imm<S>(i128, PhantomData<S>);

/// A 16-bit signed offset, encoded in 16-bits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Offset(i16);

/// A 52-bit size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct S52;
/// A 46-bit size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct S46;
/// A 40-bit size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct S40;
/// A 30-bit size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct S30;
/// A 26-bit size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct S26;
