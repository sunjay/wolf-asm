use super::Reinterpret;

impl Reinterpret<i8> for u64 {
     #[inline(always)]
     fn reinterpret(value: i8) -> Self {
         // Widen with sign-extension and then reinterpret
        Self::reinterpret(value as i64)
    }
}
