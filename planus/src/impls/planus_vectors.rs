use crate::{
    errors::{self, ErrorKind},
    slice_helpers::SliceWithStartOffset,
    traits::*,
    vectors::Vector,
};
use core::marker::PhantomData;

impl<'buf, T: VectorRead<'buf>, O> TryFrom<Vector<'buf, T>> for alloc::vec::Vec<O>
where
    O: core::convert::TryFrom<T>,
    errors::Error: From<O::Error>,
{
    type Error = crate::errors::Error;

    fn try_from(value: Vector<'buf, T>) -> Result<Self, Self::Error> {
        value
            .iter()
            .map(|v| O::try_from(v).map_err(errors::Error::from))
            .collect()
    }
    /*
    type Value = ;
    type Error = crate::errors::Error;

    fn to_owned(self) -> core::result::Result<Self::Value, Error> {
        self.iter().map(|v| v.to_owned()).collect()
    } */
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
