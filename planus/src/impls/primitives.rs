use crate::{
    builder::Builder, errors::ErrorKind, slice_helpers::SliceWithStartOffset, traits::*, Cursor,
};
use core::mem::MaybeUninit;

macro_rules! gen_primitive_types {
    ($ty:ty, $size:expr) => {
        impl Primitive for $ty {
            const ALIGNMENT: usize = $size;
            const SIZE: usize = $size;
        }

        impl WriteAsPrimitive<$ty> for $ty {
            #[inline]
            fn write<const N: usize>(&self, cursor: Cursor<'_, N>, _buffer_position: u32) {
                cursor.assert_size().finish(self.to_le_bytes());
            }
        }

        impl WriteAs<$ty> for $ty {
            type Prepared = Self;
            #[inline]
            fn prepare(&self, _builder: &mut Builder) -> Self {
                *self
            }
        }

        impl WriteAsDefault<$ty, $ty> for $ty {
            type Prepared = Self;
            #[inline]
            fn prepare(&self, _builder: &mut Builder, default: &$ty) -> Option<Self> {
                #[allow(clippy::float_cmp)]
                if self == default {
                    None
                } else {
                    Some(*self)
                }
            }
        }

        impl WriteAsOptional<$ty> for $ty {
            type Prepared = Self;
            #[inline]
            fn prepare(&self, _builder: &mut Builder) -> Option<Self> {
                Some(*self)
            }
        }

        impl<'buf> TableRead<'buf> for $ty {
            #[inline]
            fn from_buffer(
                buffer: SliceWithStartOffset<'buf>,
                offset: usize,
            ) -> core::result::Result<$ty, ErrorKind> {
                let buffer = buffer.advance_as_array(offset)?.as_array();
                Ok(<$ty>::from_le_bytes(*buffer))
            }
        }

        impl<'buf> VectorRead<'buf> for $ty {
            const STRIDE: usize = $size;
            #[inline]
            unsafe fn from_buffer(buffer: SliceWithStartOffset<'buf>, offset: usize) -> $ty {
                let buffer = buffer.unchecked_advance_as_array(offset).as_array();
                <$ty>::from_le_bytes(*buffer)
            }
        }

        impl VectorWrite<$ty> for $ty {
            const STRIDE: usize = $size;
            type Value = $ty;
            #[inline]
            fn prepare(&self, _builder: &mut Builder) -> Self::Value {
                *self
            }

            #[inline]
            unsafe fn write_values(
                values: &[$ty],
                bytes: *mut MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [MaybeUninit<u8>; $size];
                for (i, v) in values.iter().enumerate() {
                    v.write(
                        Cursor::new(&mut *bytes.add(i)),
                        buffer_position - ($size * i) as u32,
                    );
                }
            }
        }
    };
}

gen_primitive_types!(i8, 1);
gen_primitive_types!(u8, 1);
gen_primitive_types!(i16, 2);
gen_primitive_types!(u16, 2);
gen_primitive_types!(i32, 4);
gen_primitive_types!(u32, 4);
gen_primitive_types!(i64, 8);
gen_primitive_types!(u64, 8);
gen_primitive_types!(f32, 4);
gen_primitive_types!(f64, 8);
