use crate::{
    errors::{Error, ErrorKind},
    slice_helpers::SliceWithStartOffset,
    traits::*,
    vectors::Vector,
};
use core::marker::PhantomData;

impl<'buf, T: ?Sized + VectorRead<'buf>> ToOwned for Vector<'buf, T>
where
    T::Output: ToOwned,
{
    type Value = alloc::vec::Vec<<T::Output as ToOwned>::Value>;

    fn to_owned(self) -> core::result::Result<Self::Value, Error> {
        self.iter().map(|v| v.to_owned()).collect()
    }
}

impl<'buf, T: ?Sized + VectorRead<'buf>> TableRead<'buf> for Vector<'buf, T> {
    fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
    ) -> core::result::Result<Self, ErrorKind> {
        let (buffer, len) = super::array_from_buffer(buffer, offset)?;
        if len.checked_mul(T::STRIDE).ok_or(ErrorKind::InvalidLength)? <= buffer.len() {
            Ok(Vector {
                buffer,
                len,
                _marker: PhantomData,
            })
        } else {
            Err(ErrorKind::InvalidLength)
        }
    }
}
