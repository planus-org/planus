use crate::{builder::Builder, traits::*, Cursor, Offset};
use bstr::{BStr, BString};
use core::{borrow::Borrow, mem::MaybeUninit};

fn borrow(s: &BString) -> &BStr {
    s.borrow()
}

impl WriteAsOffset<str> for BString {
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Offset<str> {
        WriteAsOffset::prepare(borrow(self), builder)
    }
}

impl WriteAs<Offset<str>> for BString {
    type Prepared = Offset<str>;

    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Offset<str> {
        WriteAsOffset::prepare(borrow(self), builder)
    }
}

impl WriteAsDefault<Offset<str>, str> for BString {
    type Prepared = Offset<str>;

    #[inline]
    fn prepare(&self, builder: &mut Builder, default: &str) -> Option<Offset<str>> {
        if self == default {
            None
        } else {
            Some(WriteAsOffset::prepare(borrow(self), builder))
        }
    }
}

impl WriteAsOptional<Offset<str>> for BString {
    type Prepared = Offset<str>;

    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<Offset<str>> {
        Some(WriteAsOffset::prepare(borrow(self), builder))
    }
}

impl VectorWrite<Offset<str>> for BString {
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
