use super::{
    Reinterpret,
    slice_as_1_byte,
    slice_as_2_bytes,
    slice_as_4_bytes,
};

impl Reinterpret<i128> for u64 {
    #[inline(always)]
    fn reinterpret(value: i128) -> Self {
        // Reinterpret the lowest 8 bytes as u64
        let bytes = value.to_le_bytes();
        let bytes = slice_as_8_bytes(&bytes);
        Self::from_le_bytes(*bytes)
    }
}
