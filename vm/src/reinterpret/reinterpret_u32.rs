use super::{
    Reinterpret,
    slice_as_1_byte,
    slice_as_2_bytes,
    slice_as_4_bytes,
};

impl Reinterpret<u32> for u64 {
    #[inline(always)]
    fn reinterpret(value: u32) -> Self {
        // Widen with zero-extension
        value as u64
    }
}

impl Reinterpret<u32> for i32 {
    #[inline(always)]
    fn reinterpret(value: u32) -> Self {
        Self::from_le_bytes(value.to_le_bytes())
    }
}
