use core::mem::MaybeUninit;

use crate::{builder::Builder, traits::*, Offset, UnionVectorOffset};

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
            builder.write_with(
                T::STRIDE.checked_mul(self.len()).unwrap(),
                P::ALIGNMENT_MASK.max(u32::ALIGNMENT_MASK),
                |buffer_position, bytes| {
                    let bytes = bytes.as_mut_ptr();

                    T::write_values(&tmp, bytes, buffer_position);
                },
            );
            builder.write_with(4, 0, |_buffer_position, bytes| {
                let len = (self.len() as u32).to_le_bytes().map(MaybeUninit::new);
                bytes.copy_from_slice(&len);
            });
        }
        builder.current_offset()
    }
}

impl<T, P> WriteAs<Offset<[P]>> for [T]
where
    [T]: WriteAsOffset<[P]>,
{
    type Prepared = Offset<[P]>;

    fn prepare(&self, builder: &mut Builder) -> Offset<[P]> {
        WriteAsOffset::prepare(&self, builder)
    }
}

impl<T, P> WriteAsDefault<Offset<[P]>, ()> for [T]
where
    [T]: WriteAsOffset<[P]>,
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
    [T]: WriteAsOffset<[P]>,
{
    type Prepared = Offset<[P]>;

    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<Offset<[P]>> {
        Some(WriteAsOffset::prepare(self, builder))
    }
}

impl<T, P> WriteAsUnionVector<P> for [T]
where
    T: WriteAsUnion<P>,
{
    fn prepare(&self, builder: &mut Builder) -> UnionVectorOffset<P> {
        let mut tmp_tags: alloc::vec::Vec<MaybeUninit<u8>> =
            alloc::vec::Vec::with_capacity(self.len());
        let mut tmp_values: alloc::vec::Vec<Offset<()>> =
            alloc::vec::Vec::with_capacity(self.len());
        for v in self.iter() {
            let union_offset = v.prepare(builder);
            tmp_tags.push(MaybeUninit::new(union_offset.tag));
            tmp_values.push(union_offset.offset());
        }

        // SAFETY: TODO
        unsafe {
            builder.write_with(
                Offset::<()>::STRIDE.checked_mul(self.len()).unwrap(),
                Offset::<()>::ALIGNMENT_MASK,
                |buffer_position, bytes| {
                    let bytes = bytes.as_mut_ptr();

                    Offset::<()>::write_values(&tmp_values, bytes, buffer_position);
                },
            );
            builder.write_with(4, 0, |_buffer_position, bytes| {
                let len = (self.len() as u32).to_le_bytes().map(MaybeUninit::new);
                bytes.copy_from_slice(&len);
            });
            let values_offset = builder.current_offset();
            builder.write_with(
                self.len(),
                u32::ALIGNMENT_MASK,
                |_buffer_position, bytes| {
                    let bytes = bytes.as_mut_ptr();

                    bytes.copy_from(tmp_tags.as_ptr(), self.len());
                },
            );
            builder.write_with(4, 0, |_buffer_position, bytes| {
                let len = (self.len() as u32).to_le_bytes().map(MaybeUninit::new);
                bytes.copy_from_slice(&len);
            });
            let tags_offset = builder.current_offset();
            crate::UnionVectorOffset {
                tags_offset,
                values_offset,
                phantom: core::marker::PhantomData,
            }
        }
    }
}

impl<T, P> WriteAsDefaultUnionVector<P> for [T]
where
    T: WriteAsUnion<P>,
{
    fn prepare(&self, builder: &mut Builder) -> Option<UnionVectorOffset<P>> {
        if self.is_empty() {
            None
        } else {
            Some(WriteAsUnionVector::prepare(self, builder))
        }
    }
}

impl<T, P> WriteAsOptionalUnionVector<P> for [T]
where
    T: WriteAsUnion<P>,
{
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<UnionVectorOffset<P>> {
        Some(WriteAsUnionVector::prepare(self, builder))
    }
}
