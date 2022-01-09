use crate::traits::{VectorRead, VectorReadInner};

/*
impl<T: ToOwned, E> ToOwned for core::result::Result<T, E>
where
    errors::Error: From<E>,
{
    type Value = T::Value;

    #[inline]
    fn to_owned(self) -> crate::Result<Self::Value> {
        self?.to_owned()
    }
}
 */

impl<'buf, T: VectorReadInner<'buf>, E> VectorRead<'buf> for Result<T, E>
where
    E: core::convert::From<T::Error>,
{
    const STRIDE: usize = T::STRIDE;

    unsafe fn from_buffer(buffer: crate::SliceWithStartOffset<'buf>, offset: usize) -> Self {
        Ok(T::from_buffer(buffer, offset)?)
    }
}
