use super::{
    Reinterpret,
    slice_as_1_byte,
    slice_as_2_bytes,
    slice_as_4_bytes,
};

impl Reinterpret<i64> for u64 {
    #[inline(always)]
    fn reinterpret(value: i64) -> Self {
        Self::from_le_bytes(value.to_le_bytes())
    }
}
