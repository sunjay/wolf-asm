use super::{Reinterpret, slice_8_as_1, slice_8_as_2, slice_8_as_4};

impl Reinterpret<u64> for u128 {
    #[inline(always)]
    fn reinterpret(value: u64) -> Self {
        // Widen with zero-extension
        value as u128
    }
}

impl Reinterpret<u64> for i128 {
    #[inline(always)]
    fn reinterpret(value: u64) -> Self {
        // Widen with zero-extension (since value is always non-negative)
        value as i128
    }
}

impl Reinterpret<u64> for i64 {
    #[inline(always)]
    fn reinterpret(value: u64) -> Self {
        Self::from_le_bytes(value.to_le_bytes())
    }
}

impl Reinterpret<u64> for u32 {
    #[inline(always)]
    fn reinterpret(value: u64) -> Self {
        // Reinterpret the lowest 4 bytes as u32
        let bytes = value.to_le_bytes();
        let bytes = slice_8_as_4(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u64> for i32 {
    #[inline(always)]
    fn reinterpret(value: u64) -> Self {
        // Reinterpret the lowest 4 bytes as i32
        let bytes = value.to_le_bytes();
        let bytes = slice_8_as_4(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u64> for u16 {
    #[inline(always)]
    fn reinterpret(value: u64) -> Self {
        // Reinterpret the lowest 2 bytes as u16
        let bytes = value.to_le_bytes();
        let bytes = slice_8_as_2(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u64> for i16 {
    #[inline(always)]
    fn reinterpret(value: u64) -> Self {
        // Reinterpret the lowest 2 bytes as i16
        let bytes = value.to_le_bytes();
        let bytes = slice_8_as_2(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u64> for u8 {
    #[inline(always)]
    fn reinterpret(value: u64) -> Self {
        // Reinterpret the lowest 1 byte as u8
        let bytes = value.to_le_bytes();
        let bytes = slice_8_as_1(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u64> for i8 {
    #[inline(always)]
    fn reinterpret(value: u64) -> Self {
        // Reinterpret the lowest 1 byte as i8
        let bytes = value.to_le_bytes();
        let bytes = slice_8_as_1(&bytes);
        Self::from_le_bytes(*bytes)
    }
}
