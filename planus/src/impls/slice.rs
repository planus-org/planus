use crate::{builder::Builder, traits::*, Cursor, Offset};
use core::mem::MaybeUninit;

impl<T, P: Primitive> WriteAsOffset<[P]> for [T]
where
    T: VectorWrite<P>,
{
    fn prepare(&self, builder: &mut Builder) -> Offset<[P]> {
        let mut tmp: alloc::vec::Vec<T::Value> = alloc::vec::Vec::with_capacity(self.len());
        for v in self.iter() {
            tmp.push(v.prepare(builder));
        }
        // SAFETY: We need to make sure we always write the 4+stride*len bytes in the closure
        unsafe {
            // TODO: This will not be correctly aligned if P::ALIGNMENT_MASK is bigger than u32::ALIGNMENT_MASK
            builder.write_with(
                T::STRIDE
                    .checked_mul(self.len())
                    .unwrap()
                    .checked_add(4)
                    .unwrap(),
                P::ALIGNMENT_MASK.max(u32::ALIGNMENT_MASK),
                |buffer_position, bytes| {
                    let bytes = bytes.as_mut_ptr();

                    (self.len() as u32).write(
                        Cursor::new(&mut *(bytes as *mut [MaybeUninit<u8>; 4])),
                        buffer_position,
                    );

                    T::write_values(&tmp, bytes.add(4), buffer_position - 4);
                },
            )
        }
        builder.current_offset()
    }
}

impl<T, P> WriteAs<Offset<[P]>> for [T]
where
    P: Primitive,
    T: VectorWrite<P>,
{
    type Prepared = Offset<[P]>;

    fn prepare(&self, builder: &mut Builder) -> Offset<[P]> {
        WriteAsOffset::prepare(&self, builder)
    }
}

impl<T, P> WriteAsDefault<Offset<[P]>, ()> for [T]
where
    P: Primitive,
    T: VectorWrite<P>,
{
    type Prepared = Offset<[P]>;

    fn prepare(&self, builder: &mut Builder, _default: &()) -> Option<Offset<[P]>> {
        if self.is_empty() {
            None
        } else {
            Some(WriteAsOffset::prepare(&self, builder))
        }
    }
}

impl<T, P> WriteAsOptional<Offset<[P]>> for [T]
where
    P: Primitive,
    T: VectorWrite<P>,
{
    type Prepared = Offset<[P]>;

    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<Offset<[P]>> {
        Some(WriteAsOffset::prepare(self, builder))
    }
}
