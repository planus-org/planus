use crate::{slice_helpers::SliceWithStartOffset, traits::VectorRead};
use core::marker::PhantomData;

/// A `Vec` like view into a serialized buffer that deserializes on demand.
pub struct Vector<'buf, T: ?Sized> {
    pub(crate) buffer: SliceWithStartOffset<'buf>,
    pub(crate) len: usize,
    pub(crate) _marker: PhantomData<&'buf T>,
}

impl<'buf, T: ?Sized> Copy for Vector<'buf, T> {}
impl<'buf, T: ?Sized> Clone for Vector<'buf, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'buf, T: VectorRead<'buf> + core::fmt::Debug> core::fmt::Debug for Vector<'buf, T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: ?Sized + 'static> Vector<'static, T> {
    // Needed because beta 1.62 gave a weird error
    #[allow(unused_attributes)]
    #[doc(hidden)]
    pub const EMPTY: Self = Self {
        buffer: SliceWithStartOffset {
            buffer: &[],
            offset_from_start: 0,
        },
        len: 0,
        _marker: PhantomData,
    };
}

impl<'buf, T: ?Sized> Vector<'buf, T> {
    /// Checks if the vector is empty.
    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Returns the number of elements in the vector.
    pub fn len(self) -> usize {
        self.len
    }
}

impl<'buf, T: VectorRead<'buf>> Vector<'buf, T> {
    /// Returns the element at the given index, or None if out of bounds.
    #[inline]
    pub fn get(self, index: usize) -> Option<T> {
        if index < self.len {
            Some(unsafe { T::from_buffer(self.buffer, T::STRIDE * index) })
        } else {
            None
        }
    }

    /// Returns an iterator over the vector.
    #[inline]
    pub fn iter(self) -> VectorIter<'buf, T> {
        VectorIter(self)
    }
}

impl<'buf, T: VectorRead<'buf>> Vector<'buf, T> {
    /// Copies self into a new `Vec`.
    pub fn to_vec<O>(&self) -> crate::Result<alloc::vec::Vec<O>>
    where
        O: core::convert::TryFrom<T>,
        crate::errors::Error: From<O::Error>,
    {
        self.iter()
            .map(|v| O::try_from(v).map_err(crate::errors::Error::from))
            .collect()
    }
}

impl<'buf, T, E> Vector<'buf, core::result::Result<T, E>> {
    /// Copies self into a new `Vec`.
    pub fn to_vec_result<O>(&self) -> crate::Result<alloc::vec::Vec<O>>
    where
        T: crate::traits::VectorReadInner<'buf>,
        E: core::convert::From<T::Error>,
        O: core::convert::TryFrom<T>,
        crate::errors::Error: From<E> + From<O::Error>,
    {
        self.iter()
            .map(|v| O::try_from(v?).map_err(|e| e.into()))
            .collect()
    }
}

impl<'buf, T: VectorRead<'buf>> IntoIterator for Vector<'buf, T> {
    type Item = T;
    type IntoIter = VectorIter<'buf, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone)]
/// An iterator over the elements of a `Vector`.
pub struct VectorIter<'buf, T: ?Sized>(Vector<'buf, T>);

impl<'buf, T: VectorRead<'buf>> Iterator for VectorIter<'buf, T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        if self.0.len > 0 {
            let result = unsafe { T::from_buffer(self.0.buffer, 0) };
            self.0.buffer = self
                .0
                .buffer
                .advance(T::STRIDE)
                .expect("IMPOSSIBLE: we checked the length on creation");
            self.0.len -= 1;
            Some(result)
        } else {
            None
        }
    }
}

impl<'buf, T: VectorRead<'buf> + core::fmt::Debug> core::fmt::Debug for VectorIter<'buf, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("VectorIter").field(&self.0).finish()
    }
}
