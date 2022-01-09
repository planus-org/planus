use crate::{builder::Builder, traits::*, Offset};
use alloc::vec::Vec;

impl<T, P> WriteAsOffset<[P]> for Vec<T>
where
    P: Primitive,
    T: VectorWrite<P>,
{
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Offset<[P]> {
        WriteAsOffset::prepare(self.as_slice(), builder)
    }
}

impl<T, P> WriteAs<Offset<[P]>> for Vec<T>
where
    P: Primitive,
    T: VectorWrite<P>,
{
    type Prepared = Offset<[P]>;

    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Offset<[P]> {
        WriteAsOffset::prepare(self.as_slice(), builder)
    }
}

impl<T, P> WriteAsDefault<Offset<[P]>, ()> for Vec<T>
where
    P: Primitive,
    T: VectorWrite<P>,
{
    type Prepared = Offset<[P]>;

    #[inline]
    fn prepare(&self, builder: &mut Builder, _default: &()) -> Option<Offset<[P]>> {
        if self.is_empty() {
            None
        } else {
            Some(WriteAsOffset::prepare(self.as_slice(), builder))
        }
    }
}

impl<T, P> WriteAsOptional<Offset<[P]>> for Vec<T>
where
    P: Primitive,
    T: VectorWrite<P>,
{
    type Prepared = Offset<[P]>;

    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<Offset<[P]>> {
        Some(WriteAsOffset::prepare(self.as_slice(), builder))
    }
}
