#![deny(missing_docs)]
#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
mod backvec;
mod builder;
mod impls;
mod slice_helpers;
mod traits;

/// Error types for serialization/deserialization
pub mod errors;
/// Types for interacting with vectors in serialized data
pub mod vectors;

#[doc(hidden)]
pub extern crate alloc;
#[doc(hidden)]
pub mod table_reader;
#[doc(hidden)]
pub mod table_writer;

pub use crate::{
    builder::Builder,
    errors::Error,
    slice_helpers::{ArrayWithStartOffset, SliceWithStartOffset},
    traits::*,
    vectors::Vector,
};

#[doc(hidden)]
pub const fn check_version_compatibility(s: &str) {
    match s.as_bytes() {
        b"planus-0.3.1" => (),
        _ => panic!(
            "Your generated code is out of date, please regenerate using planus version 0.3.1"
        ),
    }
}

/// A type alias for [`Result`] with a Planus error
///
/// It is recommended to handle reading of serialized data in functions
/// returning this result type to avoid boilerplate error handling using
/// the ? operator.
///
/// [`Result`]: std::result::Result
pub type Result<T> = core::result::Result<T, Error>;
#[doc(hidden)]
pub type Cursor<'a, const N: usize> = array_init_cursor::Cursor<'a, u8, N>;

#[doc(hidden)]
pub enum Void {}

impl From<Void> for crate::Error {
    fn from(v: Void) -> Self {
        match v {}
    }
}

/// An offset to a serialized value of type T inside a buffer currently being built.
pub struct Offset<T: ?Sized> {
    offset: u32,
    phantom: core::marker::PhantomData<T>,
}
impl<T: ?Sized> Copy for Offset<T> {}
impl<T: ?Sized> Clone for Offset<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

/// An offset to a serialized union value of type T inside a buffer currently being built.
pub struct UnionOffset<T: ?Sized> {
    tag: u8,
    offset: Offset<()>,
    phantom: core::marker::PhantomData<T>,
}
impl<T: ?Sized> Copy for UnionOffset<T> {}
impl<T: ?Sized> Clone for UnionOffset<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Offset<T> {
    #[doc(hidden)]
    pub fn downcast(&self) -> Offset<()> {
        Offset {
            offset: self.offset,
            phantom: core::marker::PhantomData,
        }
    }
}

impl<T: ?Sized> UnionOffset<T> {
    #[doc(hidden)]
    #[inline]
    pub fn new(tag: u8, offset: Offset<()>) -> UnionOffset<T> {
        Self {
            tag,
            offset,
            phantom: core::marker::PhantomData,
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn tag(&self) -> u8 {
        self.tag
    }

    #[doc(hidden)]
    #[inline]
    pub fn offset(&self) -> Offset<()> {
        self.offset
    }
}
