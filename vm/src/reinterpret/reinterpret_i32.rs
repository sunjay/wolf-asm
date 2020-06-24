use super::{Reinterpret, slice_4_as_2, slice_4_as_1};

impl Reinterpret<i32> for u128 {
    #[inline(always)]
    fn reinterpret(value: i32) -> Self {
        // Widen with sign-extension and then reinterpret
        Self::reinterpret(value as i128)
    }
}

impl Reinterpret<i32> for i128 {
    #[inline(always)]
    fn reinterpret(value: i32) -> Self {
        // Widen with sign-extension
        value as i128
    }
}

impl Reinterpret<i32> for u64 {
    #[inline(always)]
    fn reinterpret(value: i32) -> Self {
        // Widen with sign-extension and then reinterpret
        Self::reinterpret(value as i64)
    }
}

impl Reinterpret<i32> for i64 {
    #[inline(always)]
    fn reinterpret(value: i32) -> Self {
        // Widen with sign-extension
        value as i64
    }
}

impl Reinterpret<i32> for u32 {
    #[inline(always)]
    fn reinterpret(value: i32) -> Self {
        Self::from_le_bytes(value.to_le_bytes())
    }
}

impl Reinterpret<i32> for u16 {
    #[inline(always)]
    fn reinterpret(value: i32) -> Self {
        // Reinterpret the lowest 2 bytes as u16
        let bytes = value.to_le_bytes();
        let bytes = slice_4_as_2(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<i32> for i16 {
    #[inline(always)]
    fn reinterpret(value: i32) -> Self {
        // Reinterpret the lowest 2 bytes as i16
        let bytes = value.to_le_bytes();
        let bytes = slice_4_as_2(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<i32> for u8 {
    #[inline(always)]
    fn reinterpret(value: i32) -> Self {
        // Reinterpret the lowest 1 byte as u8
        let bytes = value.to_le_bytes();
        let bytes = slice_4_as_1(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<i32> for i8 {
    #[inline(always)]
    fn reinterpret(value: i32) -> Self {
        // Reinterpret the lowest 1 byte as i8
        let bytes = value.to_le_bytes();
        let bytes = slice_4_as_1(&bytes);
        Self::from_le_bytes(*bytes)
    }
}
