use super::{Reinterpret, slice_2_as_1};

impl Reinterpret<i16> for u128 {
    #[inline(always)]
    fn reinterpret(value: i16) -> Self {
        // Widen with sign-extension and then reinterpret
        Self::reinterpret(value as i128)
    }
}

impl Reinterpret<i16> for i128 {
    #[inline(always)]
    fn reinterpret(value: i16) -> Self {
        // Widen with sign-extension
        value as i128
    }
}

impl Reinterpret<i16> for u64 {
    #[inline(always)]
    fn reinterpret(value: i16) -> Self {
        // Widen with sign-extension and then reinterpret
        Self::reinterpret(value as i64)
    }
}

impl Reinterpret<i16> for i64 {
    #[inline(always)]
    fn reinterpret(value: i16) -> Self {
        // Widen with sign-extension
        value as i64
    }
}

impl Reinterpret<i16> for u32 {
    #[inline(always)]
    fn reinterpret(value: i16) -> Self {
        // Widen with sign-extension and then reinterpret
        Self::reinterpret(value as i32)
    }
}

impl Reinterpret<i16> for i32 {
    #[inline(always)]
    fn reinterpret(value: i16) -> Self {
        // Widen with sign-extension
        value as i32
    }
}

impl Reinterpret<i16> for u16 {
    #[inline(always)]
    fn reinterpret(value: i16) -> Self {
        Self::from_le_bytes(value.to_le_bytes())
    }
}

impl Reinterpret<i16> for u8 {
    #[inline(always)]
    fn reinterpret(value: i16) -> Self {
        // Reinterpret the lowest 1 byte as u8
        let bytes = value.to_le_bytes();
        let bytes = slice_2_as_1(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<i16> for i8 {
    #[inline(always)]
    fn reinterpret(value: i16) -> Self {
        // Reinterpret the lowest 1 byte as i8
        let bytes = value.to_le_bytes();
        let bytes = slice_2_as_1(&bytes);
        Self::from_le_bytes(*bytes)
    }
}
