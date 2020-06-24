use super::Reinterpret;

impl Reinterpret<u8> for u64 {
    #[inline(always)]
    fn reinterpret(value: u8) -> Self {
        // Widen with zero-extension
        value as u64
    }
}

impl Reinterpret<u8> for u32 {
    #[inline(always)]
    fn reinterpret(value: u8) -> Self {
        // Widen with zero-extension
        value as u32
    }
}

impl Reinterpret<u8> for u16 {
    #[inline(always)]
    fn reinterpret(value: u8) -> Self {
        // Widen with zero-extension
        value as u16
    }
}

impl Reinterpret<u8> for i16 {
    #[inline(always)]
    fn reinterpret(value: u8) -> Self {
        // Widen with zero-extension (since value is always non-negative)
        value as i16
    }
}

impl Reinterpret<u8> for i8 {
    #[inline(always)]
    fn reinterpret(value: u8) -> Self {
        Self::from_le_bytes(value.to_le_bytes())
    }
}
