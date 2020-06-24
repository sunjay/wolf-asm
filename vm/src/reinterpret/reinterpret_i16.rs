use super::{
    Reinterpret,
    slice_as_1_byte,
    slice_as_2_bytes,
    slice_as_4_bytes,
};

impl Reinterpret<i16> for u64 {
    #[inline(always)]
    fn reinterpret(value: i16) -> Self {
        // Widen with sign-extension and then reinterpret
        Self::reinterpret(value as i64)
    }
}
