use crate::{builder::Builder, traits::*, Cursor, Offset, UnionOffset};
use core::mem::MaybeUninit;

impl<'a, P: Primitive, T: ?Sized + WriteAsPrimitive<P>> WriteAsPrimitive<P> for &'a T {
    #[inline]
    fn write<const N: usize>(&self, cursor: Cursor<'_, N>, buffer_position: u32) {
        T::write(*self, cursor, buffer_position)
    }
}

impl<'a, T1: ?Sized, T2: ?Sized + WriteAsOffset<T1>> WriteAsOffset<T1> for &'a T2 {
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Offset<T1> {
        T2::prepare(self, builder)
    }
}

impl<'a, P: Primitive, T: ?Sized + WriteAs<P>> WriteAs<P> for &'a T {
    type Prepared = T::Prepared;
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> T::Prepared {
        T::prepare(self, builder)
    }
}

impl<'a, P: Primitive, D: ?Sized, T: ?Sized + WriteAsDefault<P, D>> WriteAsDefault<P, D> for &'a T {
    type Prepared = T::Prepared;
    #[inline]
    fn prepare(&self, builder: &mut Builder, default: &D) -> Option<T::Prepared> {
        T::prepare(self, builder, default)
    }
}

impl<'a, P: Primitive, T: ?Sized + WriteAsOptional<P>> WriteAsOptional<P> for &'a T {
    type Prepared = T::Prepared;
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<T::Prepared> {
        T::prepare(self, builder)
    }
}

impl<'a, T1: ?Sized, T2: ?Sized + WriteAsUnion<T1>> WriteAsUnion<T1> for &'a T2 {
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> UnionOffset<T1> {
        T2::prepare(self, builder)
    }
}

impl<'a, T1: ?Sized, T2: ?Sized + WriteAsOptionalUnion<T1>> WriteAsOptionalUnion<T1> for &'a T2 {
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<UnionOffset<T1>> {
        T2::prepare(self, builder)
    }
}

impl<'a, P: Primitive, T: ?Sized + VectorWrite<P>> VectorWrite<P> for &'a T {
    const STRIDE: usize = T::STRIDE;
    type Value = T::Value;

    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Self::Value {
        T::prepare(self, builder)
    }

    #[inline]
    unsafe fn write_values(
        values: &[Self::Value],
        bytes: *mut MaybeUninit<u8>,
        buffer_position: u32,
    ) {
        T::write_values(values, bytes, buffer_position);
    }
}
