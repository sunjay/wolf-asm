use super::Reinterpret;

impl Reinterpret<i64> for u64 {
    #[inline(always)]
    fn reinterpret(value: i64) -> Self {
        Self::from_le_bytes(value.to_le_bytes())
    }
}
