use crate::{errors, traits::*};

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
