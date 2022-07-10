use bstr::BStr;

use crate::{builder::Builder, traits::*, Cursor, Offset};
use core::mem::MaybeUninit;

impl WriteAsOffset<str> for BStr {
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
                        (**self).as_ptr() as *const MaybeUninit<u8>,
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

impl WriteAs<Offset<str>> for BStr {
    type Prepared = Offset<str>;

    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Offset<str> {
        WriteAsOffset::prepare(self, builder)
    }
}

impl WriteAsOptional<Offset<str>> for BStr {
    type Prepared = Offset<str>;
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<Offset<str>> {
        Some(WriteAsOffset::prepare(self, builder))
    }
}

impl WriteAsDefault<Offset<str>, str> for BStr {
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

impl VectorWrite<Offset<str>> for BStr {
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
