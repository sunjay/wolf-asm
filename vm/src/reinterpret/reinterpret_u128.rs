use super::{
    Reinterpret,
    slice_16_as_8,
    slice_16_as_4,
    slice_16_as_2,
    slice_16_as_1,
};

impl Reinterpret<u128> for i128 {
    #[inline(always)]
    fn reinterpret(value: u128) -> Self {
        Self::from_le_bytes(value.to_le_bytes())
    }
}

impl Reinterpret<u128> for u64 {
    #[inline(always)]
    fn reinterpret(value: u128) -> Self {
        // Reinterpret the lowest 8 bytes as u64
        let bytes = value.to_le_bytes();
        let bytes = slice_16_as_8(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u128> for i64 {
    #[inline(always)]
    fn reinterpret(value: u128) -> Self {
        // Reinterpret the lowest 8 bytes as i64
        let bytes = value.to_le_bytes();
        let bytes = slice_16_as_8(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u128> for u32 {
    #[inline(always)]
    fn reinterpret(value: u128) -> Self {
        // Reinterpret the lowest 4 bytes as u32
        let bytes = value.to_le_bytes();
        let bytes = slice_16_as_4(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u128> for i32 {
    #[inline(always)]
    fn reinterpret(value: u128) -> Self {
        // Reinterpret the lowest 4 bytes as i32
        let bytes = value.to_le_bytes();
        let bytes = slice_16_as_4(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u128> for u16 {
    #[inline(always)]
    fn reinterpret(value: u128) -> Self {
        // Reinterpret the lowest 2 bytes as u16
        let bytes = value.to_le_bytes();
        let bytes = slice_16_as_2(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u128> for i16 {
    #[inline(always)]
    fn reinterpret(value: u128) -> Self {
        // Reinterpret the lowest 2 bytes as i16
        let bytes = value.to_le_bytes();
        let bytes = slice_16_as_2(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u128> for u8 {
    #[inline(always)]
    fn reinterpret(value: u128) -> Self {
        // Reinterpret the lowest 1 byte as u8
        let bytes = value.to_le_bytes();
        let bytes = slice_16_as_1(&bytes);
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u128> for i8 {
    #[inline(always)]
    fn reinterpret(value: u128) -> Self {
        // Reinterpret the lowest 1 byte as i8
        let bytes = value.to_le_bytes();
        let bytes = slice_16_as_1(&bytes);
        Self::from_le_bytes(*bytes)
    }
}
