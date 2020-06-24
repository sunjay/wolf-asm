use super::Reinterpret;

impl Reinterpret<i32> for u64 {
    #[inline(always)]
    fn reinterpret(value: i32) -> Self {
        // Widen with sign-extension and then reinterpret
        Self::reinterpret(value as i64)
    }
}
