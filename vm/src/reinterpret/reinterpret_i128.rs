use super::{Reinterpret, slice_16_as_8};

impl Reinterpret<i128> for u64 {
    #[inline(always)]
    fn reinterpret(value: i128) -> Self {
        // Reinterpret the lowest 8 bytes as u64
        let bytes = value.to_le_bytes();
        let bytes = slice_16_as_8(&bytes);
        Self::from_le_bytes(*bytes)
    }
}
