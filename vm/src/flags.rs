/// The carry flag
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CF {
    NoCarry = 0,
    Carry = 1,
}

/// The zero flag
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZF {
    NonZero = 0,
    Zero = 1,
}

/// The sign flag
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SF {
    PositiveSign = 0,
    NegativeSign = 1,
}

/// The overflow flag
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OF {
    NoOverflow = 0,
    Overflow = 1,
}

/// The status/flags register
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flags {
    pub carry: CF,
    pub zero: ZF,
    pub sign: SF,
    pub overflow: OF,
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            carry: CF::NoCarry,
            zero: ZF::Zero,
            sign: SF::PositiveSign,
            overflow: OF::NoOverflow,
        }
    }
}
