use crate::{
    builder::Builder, errors::ErrorKind, slice_helpers::SliceWithStartOffset, Cursor, Offset,
    Result, UnionOffset,
};
use core::mem::MaybeUninit;

#[doc(hidden)]
pub trait Primitive {
    const ALIGNMENT: usize;
    const ALIGNMENT_MASK: usize = Self::ALIGNMENT - 1;
    const SIZE: usize;
}

pub trait ReadAsRoot<'a>: Sized {
    fn read_as_root(slice: &'a [u8]) -> Result<Self>;
}

pub trait WriteAs<P: Primitive> {
    #[doc(hidden)]
    type Prepared: WriteAsPrimitive<P>;
    #[doc(hidden)]
    fn prepare(&self, builder: &mut Builder) -> Self::Prepared;
}

pub trait WriteAsDefault<P: Primitive, D: ?Sized> {
    #[doc(hidden)]
    type Prepared: WriteAsPrimitive<P>;
    #[doc(hidden)]
    fn prepare(&self, builder: &mut Builder, default: &D) -> Option<Self::Prepared>;
}

pub trait WriteAsOptional<P: Primitive> {
    #[doc(hidden)]
    type Prepared: WriteAsPrimitive<P>;
    #[doc(hidden)]
    fn prepare(&self, builder: &mut Builder) -> Option<Self::Prepared>;
}

pub trait WriteAsOffset<T: ?Sized> {
    #[doc(hidden)]
    fn prepare(&self, builder: &mut Builder) -> Offset<T>;
}

pub trait WriteAsUnion<T: ?Sized> {
    #[doc(hidden)]
    fn prepare(&self, builder: &mut Builder) -> UnionOffset<T>;
}

pub trait WriteAsOptionalUnion<T: ?Sized> {
    #[doc(hidden)]
    fn prepare(&self, builder: &mut Builder) -> Option<UnionOffset<T>>;
}

#[doc(hidden)]
pub trait WriteAsPrimitive<P> {
    fn write<const N: usize>(&self, cursor: Cursor<'_, N>, buffer_position: u32);
}

#[doc(hidden)]
pub trait TableRead<'buf>: Sized {
    fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
    ) -> core::result::Result<Self, ErrorKind>;
}

#[doc(hidden)]
pub trait TableReadUnion<'buf>: Sized {
    fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
        tag: u8,
    ) -> core::result::Result<Self, ErrorKind>;
}

pub trait VectorRead<'buf> {
    #[doc(hidden)]
    const STRIDE: usize;
    #[doc(hidden)]
    unsafe fn from_buffer(buffer: SliceWithStartOffset<'buf>, offset: usize) -> Self;
}

/// This trait is a hack to get around the coherence restriction.
/// Ideally we would want to be able to do an `impl VectorRead<'buf> for planus::Result<MyType>`
/// in our generated code, however instead we do something like this:
///   impl<T: VectorReadInner<'buf>, E> VectorRead<'buf> for Result<T, E>
#[doc(hidden)]
pub trait VectorReadInner<'buf>: Sized {
    #[doc(hidden)]
    type Error: Sized;
    #[doc(hidden)]
    const STRIDE: usize;
    #[doc(hidden)]
    unsafe fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
    ) -> core::result::Result<Self, Self::Error>;
}

pub trait VectorWrite<P> {
    #[doc(hidden)]
    const STRIDE: usize;
    #[doc(hidden)]
    type Value: WriteAsPrimitive<P> + Sized;
    #[doc(hidden)]
    fn prepare(&self, builder: &mut Builder) -> Self::Value;
    #[doc(hidden)]
    unsafe fn write_values(
        values: &[Self::Value],
        bytes: *mut MaybeUninit<u8>,
        buffer_position: u32,
    );
}
