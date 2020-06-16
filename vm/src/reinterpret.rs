/// A trait that allows the bits of a type to be reinterpreted as another type.
/// Unlike a conversion or cast, this operation does not attempt to preserve the
/// value being reinterpreted.
///
/// All possible implementations of this trait are not provided.
///
/// # Implementation Invariants
///
/// While this trait may not always preserve value, it is important that it
/// preserves signedness during a widening conversion from a signed value to a
/// signed value.
///
/// Similarly, a widening or narrowing conversion between two values of the same
/// signedness should preserve the value if the value is in the range
/// representable by the narrower type.
///
/// Examples:
/// * `12u64` preserves its value when reinterpreted as: u64, i64, u32, i32,
///   u16, i16, u8, i8
/// * `-14i64` preserves its value when reinterpreted as: u64, u32, u16, u8
/// * `27i8` preserves its value when reinterpreted as: i64, u64
///   (widening conversion: any wider integer would preserve the value)
///
/// Performance: An implementation of this trait must not panic or perform any
/// validation whatsoever.
pub trait Reinterpret<T>: Copy {
    fn reinterpret(value: T) -> Self;
}

impl<T: Copy> Reinterpret<T> for T {
    #[inline(always)]
    fn reinterpret(value: T) -> Self {
        value
    }
}

impl Reinterpret<u64> for i64 {
    #[inline(always)]
    fn reinterpret(value: u64) -> Self {
        Self::from_le_bytes(value.to_le_bytes())
    }
}

impl Reinterpret<u64> for u32 {
    #[inline(always)]
    fn reinterpret(value: u64) -> Self {
        // Reinterpret the lowest 4 bytes as u32
        let bytes = value.to_le_bytes();
        // Safety: u64 is at least 8 bytes, which is more than 4 bytes
        let bytes = unsafe { slice_as_4_bytes(bytes.get_unchecked(..4)) };
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u64> for i32 {
    #[inline(always)]
    fn reinterpret(value: u64) -> Self {
        // Reinterpret the lowest 4 bytes as i32
        let bytes = value.to_le_bytes();
        // Safety: u64 is at least 8 bytes, which is more than 4 bytes
        let bytes = unsafe { slice_as_4_bytes(bytes.get_unchecked(..4)) };
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u64> for u16 {
    #[inline(always)]
    fn reinterpret(value: u64) -> Self {
        // Reinterpret the lowest 2 bytes as u16
        let bytes = value.to_le_bytes();
        // Safety: u64 is at least 8 bytes, which is more than 2 bytes
        let bytes = unsafe { slice_as_2_bytes(bytes.get_unchecked(..2)) };
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u64> for i16 {
    #[inline(always)]
    fn reinterpret(value: u64) -> Self {
        // Reinterpret the lowest 2 bytes as i16
        let bytes = value.to_le_bytes();
        // Safety: u64 is at least 8 bytes, which is more than 2 bytes
        let bytes = unsafe { slice_as_2_bytes(bytes.get_unchecked(..2)) };
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u64> for u8 {
    #[inline(always)]
    fn reinterpret(value: u64) -> Self {
        // Reinterpret the lowest 1 byte as u8
        let bytes = value.to_le_bytes();
        // Safety: u64 is at least 8 bytes, which is more than 1 byte
        let bytes = unsafe { slice_as_1_byte(bytes.get_unchecked(..1)) };
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<u64> for i8 {
    #[inline(always)]
    fn reinterpret(value: u64) -> Self {
        // Reinterpret the lowest 1 byte as i8
        let bytes = value.to_le_bytes();
        // Safety: u64 is at least 8 bytes, which is more than 1 byte
        let bytes = unsafe { slice_as_1_byte(bytes.get_unchecked(..1)) };
        Self::from_le_bytes(*bytes)
    }
}

impl Reinterpret<i64> for u64 {
    #[inline(always)]
    fn reinterpret(value: i64) -> Self {
        Self::from_le_bytes(value.to_le_bytes())
    }
}

impl Reinterpret<i32> for u64 {
    #[inline(always)]
    fn reinterpret(value: i32) -> Self {
        // Widen with sign-extension and then reinterpret
        Self::reinterpret(value as i64)
    }
}

impl Reinterpret<i16> for u64 {
    #[inline(always)]
    fn reinterpret(value: i16) -> Self {
        // Widen with sign-extension and then reinterpret
        Self::reinterpret(value as i64)
    }
}

impl Reinterpret<i8> for u64 {
    #[inline(always)]
    fn reinterpret(value: i8) -> Self {
        // Widen with sign-extension and then reinterpret
        Self::reinterpret(value as i64)
    }
}

impl Reinterpret<u32> for u64 {
    #[inline(always)]
    fn reinterpret(value: u32) -> Self {
        // Widen with zero-extension
        value as u64
    }
}

impl Reinterpret<u16> for u64 {
    #[inline(always)]
    fn reinterpret(value: u16) -> Self {
        // Widen with zero-extension
        value as u64
    }
}

impl Reinterpret<u8> for u64 {
    #[inline(always)]
    fn reinterpret(value: u8) -> Self {
        // Widen with zero-extension
        value as u64
    }
}

impl Reinterpret<i128> for u64 {
    #[inline(always)]
    fn reinterpret(value: i128) -> Self {
        // Reinterpret the lowest 8 bytes as u64
        let bytes = value.to_le_bytes();
        // Safety: i128 is at least 16 bytes, which is more than 8 bytes
        let bytes = unsafe { slice_as_8_bytes(bytes.get_unchecked(..8)) };
        Self::from_le_bytes(*bytes)
    }
}

/// Reinterprets a slice as a slice of a specific size
///
/// # Safety
///
/// The input slice must be the exact length of the output slice.
#[inline(always)]
unsafe fn slice_as_8_bytes(bytes: &[u8]) -> &[u8; 8] {
    let ptr = bytes.as_ptr() as *const [u8; 8];
    &*ptr
}

/// Reinterprets a slice as a slice of a specific size
///
/// # Safety
///
/// The input slice must be the exact length of the output slice.
#[inline(always)]
unsafe fn slice_as_4_bytes(bytes: &[u8]) -> &[u8; 4] {
    let ptr = bytes.as_ptr() as *const [u8; 4];
    &*ptr
}

/// Reinterprets a slice as a slice of a specific size
///
/// # Safety
///
/// The input slice must be the exact length of the output slice.
#[inline(always)]
unsafe fn slice_as_2_bytes(bytes: &[u8]) -> &[u8; 2] {
    let ptr = bytes.as_ptr() as *const [u8; 2];
    &*ptr
}

/// Reinterprets a slice as a slice of a specific size
///
/// # Safety
///
/// The input slice must be the exact length of the output slice.
#[inline(always)]
unsafe fn slice_as_1_byte(bytes: &[u8]) -> &[u8; 1] {
    let ptr = bytes.as_ptr() as *const [u8; 1];
    &*ptr
}
