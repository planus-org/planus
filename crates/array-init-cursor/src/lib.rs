//! Small crate for initializing an uninitialized slice
#![no_std]

use core::mem::MaybeUninit;

mod util;

/// A fixed-size cursor for initializing [`MaybeUninit`] arrays
///
/// The cursor will guarantee that all values have been
/// initialized when the value is dropped, which means
/// that it is safe to call [`MaybeUninit::assume_init()`].
///
/// **NOTE:** This guarantee only holds as long as [`Drop::drop()`] is called.
///           If the value goes out of scope without drop being called (e.g. because
///           of [`core::mem::forget()`]), then this guarantee no longer applies.
pub struct Cursor<'a, T, const N: usize> {
    slice: &'a mut [MaybeUninit<T>; N],
}

impl<'a, T, const N: usize> Cursor<'a, T, N> {
    /// Creates a new cursor.
    pub fn new(slice: &'a mut [MaybeUninit<T>; N]) -> Self {
        Self { slice }
    }

    fn write_impl(&mut self, value: [T; N]) {
        *self.slice = unsafe {
            let ptr = &value as *const [T; N] as *const [MaybeUninit<T>; N];
            let read_value = core::ptr::read(ptr);
            core::mem::drop(value);
            read_value
        };
    }

    /// Finishes the buffer by writing the remaining values.
    ///
    /// This is equivalent to calling [`self.write::<N, 0>(value)`](`Self::write`), except it is slightly
    /// more ergonomic.
    pub fn finish(mut self, value: [T; N]) {
        self.write_impl(value);
        core::mem::forget(self);
    }

    /// Writes `L` values to the buffer and returns a new cursor for the remaining `R` values.
    ///
    /// This function cannot compile unless `L + R == N`, however it will be able to pass through
    /// `cargo check`, since the error is not discovered by `rustc` until it tries to instantiate
    /// the code.
    pub fn write<const L: usize, const R: usize>(self, value: [T; L]) -> Cursor<'a, T, R> {
        let (l, r) = self.split::<L, R>();
        l.finish(value);
        r
    }

    unsafe fn into_buf(self) -> &'a mut [MaybeUninit<T>; N] {
        core::mem::transmute(self)
    }

    /// Splits the cursor in two.
    ///
    /// This function cannot compile unless `L + R == N`, however it will be able to pass through
    /// `cargo check`, since the error is not discovered by `rustc` until it tries to instantiate
    /// the code.
    pub fn split<const L: usize, const R: usize>(self) -> (Cursor<'a, T, L>, Cursor<'a, T, R>) {
        let buf = unsafe { self.into_buf() };
        let (l, r) = crate::util::split_mut::<_, N, L, R>(buf);
        (Cursor { slice: l }, Cursor { slice: r })
    }

    /// Compile-time assertion that `N == M` to work-around limitations in rust generics.
    ///
    /// This is useful if a type-signature requires the function to have a generic size
    /// argument, but you want compile-time errors when called with the wrong parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// fn example<const N: usize>(cursor: array_init_cursor::Cursor<'_, u8, N>) {
    ///     let cursor: array_init_cursor::Cursor<u8, 10> = cursor.assert_size();
    /// }
    /// ```
    pub fn assert_size<const M: usize>(self) -> Cursor<'a, T, M> {
        let (l, _) = self.split::<M, 0>();
        l
    }
}

impl<'a, T, const N: usize> Drop for Cursor<'a, T, N> {
    /// Will panic unless cursor has been completely initialized
    fn drop(&mut self) {
        if N > 0 {
            panic!("Cursor still has uninitialized bytes");
        }
    }
}
