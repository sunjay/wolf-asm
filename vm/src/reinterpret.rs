mod reinterpret_i128;
mod reinterpret_i16;
mod reinterpret_i32;
mod reinterpret_i64;
mod reinterpret_i8;
mod reinterpret_u128;
mod reinterpret_u16;
mod reinterpret_u32;
mod reinterpret_u64;
mod reinterpret_u8;

/// A trait that allows the bits of a type to be reinterpreted as another type.
/// Unlike a conversion or cast, this operation does not attempt to preserve the
/// value being reinterpreted.
///
/// All possible implementations of this trait are not provided.
///
/// # Implementation Invariants
///
/// While this trait may not always preserve value, it is important that it
/// preserves signedness during a widening conversion from a signed value.
///
/// Similarly, a widening or narrowing conversion from a signed value should
/// preserve the value if the value is in the range representable by the
/// narrower type.
///
/// Examples:
/// * `12u64` preserves its value when reinterpreted as: u64, i64, u32, i32,
///   u16, i16, u8, i8 because it is in the range of all of those types
/// * `-14i64` preserves its value when reinterpreted as: i64, i32, i16, i8
///   because it is in the range of all of those types
/// * `27i8` preserves its value when reinterpreted as: u64, i64, u32, i32,
///   u16, i16, u8, i8 because it is in the range of all of those types
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

/// Reinterprets a slice as a smaller slice
#[inline(always)]
fn slice_as_8_bytes(bytes: &[u8; 16]) -> &[u8; 8] {
    // Safety: 16 > 8, so we can definitely expect this to be a valid cast/slice
    let ptr = bytes.as_ptr() as *const [u8; 8];
    unsafe { &*ptr }
}

/// Reinterprets a slice as a smaller slice
#[inline(always)]
fn slice_as_4_bytes(bytes: &[u8; 8]) -> &[u8; 4] {
    // Safety: 8 > 4, so we can definitely expect this to be a valid cast/slice
    let ptr = bytes.as_ptr() as *const [u8; 4];
    unsafe { &*ptr }
}

/// Reinterprets a slice as a smaller slice
#[inline(always)]
fn slice_as_2_bytes(bytes: &[u8; 8]) -> &[u8; 2] {
    // Safety: 8 > 2, so we can definitely expect this to be a valid cast/slice
    let ptr = bytes.as_ptr() as *const [u8; 2];
    unsafe { &*ptr }
}

/// Reinterprets a slice as a smaller slice
#[inline(always)]
fn slice_as_1_byte(bytes: &[u8; 8]) -> &[u8; 1] {
    // Safety: 8 > 1, so we can definitely expect this to be a valid cast/slice
    let ptr = bytes.as_ptr() as *const [u8; 1];
    unsafe { &*ptr }
}
