use super::{Reinterpret, slice_2_as_1};

impl Reinterpret<u16> for u128 {
    #[inline(always)]
    fn reinterpret(value: u16) -> Self {
        // Widen with zero-extension
        value as u128
    }
}

impl Reinterpret<u16> for i128 {
    #[inline(always)]
    fn reinterpret(value: u16) -> Self {
        // Widen with zero-extension (since value is always non-negative)
        value as i128
    }
}

impl Reinterpret<u16> for u64 {
    #[inline(always)]
    fn reinterpret(value: u16) -> Self {
        // Widen with zero-extension
        value as u64
    }
}

impl Reinterpret<u16> for i64 {
    #[inline(always)]
    fn reinterpret(value: u16) -> Self {
        // Widen with zero-extension (since value is always non-negative)
        value as i64
    }
}

impl Reinterpret<u16> for u32 {
    #[inline(always)]
    fn reinterpret(value: u16) -> Self {
        // Widen with zero-extension
        value as u32
    }
}

impl Reinterpret<u16> for i32 {
    #[inline(always)]
    fn reinterpret(value: u16) -> Self {
        // Widen with zero-extension (since value is always non-negative)
        value as i32
    }
}

impl Reinterpret<u16> for i16 {
    #[inline(always)]
    fn reinterpret(value: u16) -> Self {
        Self::from_le_bytes(value.to_le_bytes())
    }
}

impl Reinterpret<u16> for u8 {
    #[inline(always)]
    fn reinterpret(value: u16) -> Self {
        // Reinterpret the lowest 1 byte as u8
        let bytes = value.to_le_bytes();
        let bytes = slice_2_as_1(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u16> for i8 {
    #[inline(always)]
    fn reinterpret(value: u16) -> Self {
        // Reinterpret the lowest 1 byte as i8
        let bytes = value.to_le_bytes();
        let bytes = slice_2_as_1(&bytes);
        Self::from_le_bytes(*bytes)
    }
}
