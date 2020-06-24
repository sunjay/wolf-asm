use super::{
    Reinterpret,
    slice_as_1_byte,
    slice_as_2_bytes,
    slice_as_4_bytes,
};

impl Reinterpret<u16> for u64 {
    #[inline(always)]
    fn reinterpret(value: u16) -> Self {
        // Widen with zero-extension
        value as u64
    }
}

impl Reinterpret<u16> for u32 {
    #[inline(always)]
    fn reinterpret(value: u16) -> Self {
        // Widen with zero-extension
        value as u32
    }
}

impl Reinterpret<u16> for i16 {
    #[inline(always)]
    fn reinterpret(value: u16) -> Self {
        Self::from_le_bytes(value.to_le_bytes())
    }
}
