use crate::{
    builder::Builder, errors::ErrorKind, slice_helpers::SliceWithStartOffset, traits::*, Cursor,
};
use core::mem::MaybeUninit;

impl Primitive for bool {
    const ALIGNMENT: usize = 1;
    const SIZE: usize = 1;
}

impl WriteAsPrimitive<bool> for bool {
    #[inline]
    fn write<const N: usize>(&self, cursor: Cursor<'_, N>, _buffer_position: u32) {
        cursor.assert_size().finish([if *self { 1 } else { 0 }]);
    }
}

impl WriteAs<bool> for bool {
    type Prepared = Self;
    #[inline]
    fn prepare(&self, _builder: &mut Builder) -> Self {
        *self
    }
}

impl WriteAsDefault<bool, bool> for bool {
    type Prepared = Self;
    #[inline]
    fn prepare(&self, _builder: &mut Builder, default: &bool) -> Option<bool> {
        if self == default {
            None
        } else {
            Some(*self)
        }
    }
}

impl WriteAsOptional<bool> for bool {
    type Prepared = Self;
    #[inline]
    fn prepare(&self, _builder: &mut Builder) -> Option<Self> {
        Some(*self)
    }
}

impl<'buf> TableRead<'buf> for bool {
    #[inline]
    fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
    ) -> core::result::Result<bool, ErrorKind> {
        Ok(buffer.advance_as_array::<1>(offset)?.as_array()[0] != 0)
    }
}

impl<'buf> VectorRead<'buf> for bool {
    const STRIDE: usize = 1;

    #[inline]
    unsafe fn from_buffer(buffer: SliceWithStartOffset<'buf>, offset: usize) -> bool {
        *buffer.as_slice().get_unchecked(offset) != 0
    }
}

impl VectorWrite<bool> for bool {
    const STRIDE: usize = 1;

    type Value = bool;

    #[inline]
    fn prepare(&self, _builder: &mut Builder) -> Self::Value {
        *self
    }

    #[inline]
    unsafe fn write_values(
        values: &[Self::Value],
        bytes: *mut MaybeUninit<u8>,
        buffer_position: u32,
    ) {
        let bytes = bytes as *mut [MaybeUninit<u8>; 1];
        for (i, v) in values.iter().enumerate() {
            v.write(Cursor::new(&mut *bytes.add(i)), buffer_position - i as u32);
        }
    }
}
