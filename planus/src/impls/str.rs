use crate::{
    builder::Builder, errors::ErrorKind, slice_helpers::SliceWithStartOffset, traits::*, Cursor,
    Offset,
};
use core::mem::MaybeUninit;

impl WriteAsOffset<str> for str {
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Offset<str> {
        let size_including_len_and_null = self.len().checked_add(5).unwrap();
        // SAFETY: We make sure to write the 4+len+1 bytes inside the closure.
        unsafe {
            builder.write_with(
                size_including_len_and_null,
                u32::ALIGNMENT_MASK,
                |buffer_position, bytes| {
                    let bytes = bytes.as_mut_ptr();

                    (self.len() as u32).write(
                        Cursor::new(&mut *(bytes as *mut [MaybeUninit<u8>; 4])),
                        buffer_position,
                    );
                    core::ptr::copy_nonoverlapping(
                        self.as_bytes().as_ptr() as *const MaybeUninit<u8>,
                        bytes.add(4),
                        self.len(),
                    );
                    bytes.add(4 + self.len()).write(MaybeUninit::new(0));
                },
            )
        }
        builder.current_offset()
    }
}

impl WriteAs<Offset<str>> for str {
    type Prepared = Offset<str>;

    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Offset<str> {
        WriteAsOffset::prepare(self, builder)
    }
}

impl WriteAsOptional<Offset<str>> for str {
    type Prepared = Offset<str>;
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<Offset<str>> {
        Some(WriteAsOffset::prepare(self, builder))
    }
}

impl WriteAsDefault<Offset<str>, str> for str {
    type Prepared = Offset<str>;

    #[inline]
    fn prepare(&self, builder: &mut Builder, default: &str) -> Option<Offset<str>> {
        if self == default {
            None
        } else {
            Some(WriteAsOffset::prepare(self, builder))
        }
    }
}

impl<'buf> TableRead<'buf> for &'buf str {
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
        Ok(core::str::from_utf8(slice)?)
    }
}

impl<'buf> VectorReadInner<'buf> for &'buf str {
    type Error = crate::errors::Error;

    const STRIDE: usize = 4;
    #[inline]
    unsafe fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
    ) -> crate::Result<&'buf str> {
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
        let str = core::str::from_utf8(slice)
            .map_err(|source| ErrorKind::InvalidUtf8 { source })
            .map_err(add_context)?;
        Ok(str)
    }
}

impl VectorWrite<Offset<str>> for str {
    type Value = Offset<str>;

    const STRIDE: usize = 4;
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Self::Value {
        WriteAs::prepare(self, builder)
    }

    #[inline]
    unsafe fn write_values(
        values: &[Offset<str>],
        bytes: *mut MaybeUninit<u8>,
        buffer_position: u32,
    ) {
        let bytes = bytes as *mut [MaybeUninit<u8>; 4];
        for (i, v) in values.iter().enumerate() {
            v.write(
                Cursor::new(&mut *bytes.add(i)),
                buffer_position - (4 * i) as u32,
            );
        }
    }
}
