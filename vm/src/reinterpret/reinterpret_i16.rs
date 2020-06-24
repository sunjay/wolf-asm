use super::Reinterpret;

impl Reinterpret<i16> for u64 {
    #[inline(always)]
    fn reinterpret(value: i16) -> Self {
        // Widen with sign-extension and then reinterpret
        Self::reinterpret(value as i64)
    }
}
