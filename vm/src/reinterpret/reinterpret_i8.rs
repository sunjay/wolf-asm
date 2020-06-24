use super::Reinterpret;

impl Reinterpret<i8> for u128 {
    #[inline(always)]
    fn reinterpret(value: i8) -> Self {
        // Widen with sign-extension and then reinterpret
        Self::reinterpret(value as i128)
    }
}

impl Reinterpret<i8> for i128 {
    #[inline(always)]
    fn reinterpret(value: i8) -> Self {
        // Widen with sign-extension
        value as i128
    }
}

impl Reinterpret<i8> for u64 {
    #[inline(always)]
    fn reinterpret(value: i8) -> Self {
        // Widen with sign-extension and then reinterpret
        Self::reinterpret(value as i64)
    }
}

impl Reinterpret<i8> for i64 {
    #[inline(always)]
    fn reinterpret(value: i8) -> Self {
        // Widen with sign-extension
        value as i64
    }
}

impl Reinterpret<i8> for u32 {
    #[inline(always)]
    fn reinterpret(value: i8) -> Self {
        // Widen with sign-extension and then reinterpret
        Self::reinterpret(value as i32)
    }
}

impl Reinterpret<i8> for i32 {
    #[inline(always)]
    fn reinterpret(value: i8) -> Self {
        // Widen with sign-extension
        value as i32
    }
}

impl Reinterpret<i8> for u16 {
    #[inline(always)]
    fn reinterpret(value: i8) -> Self {
        // Widen with sign-extension and then reinterpret
        Self::reinterpret(value as i16)
    }
}

impl Reinterpret<i8> for i16 {
    #[inline(always)]
    fn reinterpret(value: i8) -> Self {
        // Widen with sign-extension
        value as i16
    }
}

impl Reinterpret<i8> for u8 {
    #[inline(always)]
    fn reinterpret(value: i8) -> Self {
        Self::from_le_bytes(value.to_le_bytes())
    }
}
