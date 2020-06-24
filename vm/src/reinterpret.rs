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

// Source: https://docs.rs/static_assertions/1.1.0/static_assertions/macro.const_assert.html
#[macro_export]
macro_rules! const_assert {
    ($x:expr $(,)?) => {
        #[allow(unknown_lints, eq_op)]
        const _: [(); 0 - !{ const ASSERT: bool = $x; ASSERT } as usize] = [];
    };
}

macro_rules! reinterpret_slice {
    ($(fn $fname:ident(&[T; $inp:literal]) -> &[T; $out:literal];)*) => {
        $(
            /// Reinterprets a slice as a smaller slice
            #[inline(always)]
            fn $fname<T>(bytes: &[T; $inp]) -> &[T; $out] {
                // Safety: if the size of the input type is greater than the
                // size of the output type, this is a valid cast/slice
                const_assert!($inp > $out);
                let ptr = bytes.as_ptr() as *const [T; $out];
                unsafe { &*ptr }
            }
        )*
    };
}

reinterpret_slice! {
    fn slice_16_as_8(&[T; 16]) -> &[T; 8];
    fn slice_16_as_4(&[T; 16]) -> &[T; 4];
    fn slice_16_as_2(&[T; 16]) -> &[T; 2];
    fn slice_16_as_1(&[T; 16]) -> &[T; 1];
    fn slice_8_as_4(&[T; 8]) -> &[T; 4];
    fn slice_8_as_2(&[T; 8]) -> &[T; 2];
    fn slice_8_as_1(&[T; 8]) -> &[T; 1];
}
