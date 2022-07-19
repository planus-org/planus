use crate::{errors::ErrorKind, slice_helpers::SliceWithStartOffset, traits::*};

impl<'buf> TableRead<'buf> for &'buf [u8] {
    fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
    ) -> core::result::Result<Self, ErrorKind> {
        let (buffer, len) = super::array_from_buffer(buffer, offset)?;
        buffer.as_slice().get(..len).ok_or(ErrorKind::InvalidLength)
    }
}

impl<'buf> TableRead<'buf> for &'buf [i8] {
    fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
    ) -> core::result::Result<Self, ErrorKind> {
        let (buffer, len) = super::array_from_buffer(buffer, offset)?;
        let slice = buffer
            .as_slice()
            .get(..len)
            .ok_or(ErrorKind::InvalidLength)?;
        Ok(unsafe { core::slice::from_raw_parts(slice.as_ptr() as *const i8, slice.len()) })
    }
}
