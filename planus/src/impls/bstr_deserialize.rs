use bstr::BStr;

use crate::{errors::ErrorKind, slice_helpers::SliceWithStartOffset, traits::*};

impl<'buf> TableRead<'buf> for &'buf BStr {
    fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
    ) -> core::result::Result<Self, ErrorKind> {
        let (buffer, len) = super::array_from_buffer(buffer, offset)?;
        #[cfg(feature = "extra-validation")]
        if buffer.as_slice().get(len) != Some(&0) {
            return Err(ErrorKind::MissingNullTerminator);
        }
        let slice = buffer
            .as_slice()
            .get(..len)
            .ok_or(ErrorKind::InvalidLength)?;
        Ok(slice.into())
    }
}

impl<'buf> VectorReadInner<'buf> for &'buf BStr {
    type Error = crate::errors::Error;

    const STRIDE: usize = 4;
    #[inline]
    unsafe fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
    ) -> crate::Result<&'buf BStr> {
        let add_context =
            |e: ErrorKind| e.with_error_location("[str]", "get", buffer.offset_from_start);
        let (slice, len) = super::array_from_buffer(buffer, offset).map_err(add_context)?;
        #[cfg(feature = "extra-validation")]
        if slice.as_slice().get(len) != Some(&0) {
            return Err(add_context(ErrorKind::MissingNullTerminator));
        }
        let slice = slice
            .as_slice()
            .get(..len)
            .ok_or(ErrorKind::InvalidLength)
            .map_err(add_context)?;
        Ok(slice.into())
    }
}
