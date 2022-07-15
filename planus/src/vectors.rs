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

impl<'buf, T: ?Sized> Vector<'buf, T> {
    /// Returns an empty `Vector`
    #[inline]
    #[must_use]
    pub const fn empty() -> Vector<'buf, T> {
        Self {
            buffer: SliceWithStartOffset {
                buffer: &[],
                offset_from_start: 0,
            },
            len: 0,
            _marker: PhantomData,
        }
    }

    /// Checks if the vector is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Returns the number of elements in the vector.
    #[inline]
    #[must_use]
    pub fn len(self) -> usize {
        self.len
    }
}

impl<'buf, T: VectorRead<'buf>> Vector<'buf, T> {
    /// Returns the first element of the `Vector`, or `None` if it is empty.
    #[inline]
    #[must_use]
    pub fn first(self) -> Option<T> {
        self.get(0)
    }

    /// Returns the last element of the `Vector`, or `None` if it is empty.
    #[inline]
    #[must_use]
    pub fn last(self) -> Option<T> {
        self.get(self.len().checked_sub(1)?)
    }

    /// Returns the element at the given index, or None if out of bounds.
    #[inline]
    #[must_use]
    pub fn get(self, index: usize) -> Option<T> {
        if index < self.len {
            Some(unsafe { T::from_buffer(self.buffer, T::STRIDE * index) })
        } else {
            None
        }
    }

    /// Returns an iterator over the vector.
    #[inline]
    #[must_use]
    pub fn iter(self) -> VectorIter<'buf, T> {
        VectorIter(self)
    }

    /// Returns the first and all the rest of the elements of the `Vector`, or `None` if it is empty
    #[inline]
    #[must_use]
    pub fn split_first(self) -> Option<(T, Vector<'buf, T>)> {
        if self.is_empty() {
            None
        } else {
            let value = unsafe { T::from_buffer(self.buffer, 0) };
            let vector = Self {
                buffer: self
                    .buffer
                    .advance(T::STRIDE)
                    .expect("IMPOSSIBLE: the length was checked on creation"),
                len: self.len - 1,
                _marker: PhantomData,
            };
            Some((value, vector))
        }
    }

    /// Returns the last and all the rest of the elements of the `Vector`, or `None` if it is empty
    #[inline]
    #[must_use]
    pub fn split_last(self) -> Option<(T, Vector<'buf, T>)> {
        if self.is_empty() {
            None
        } else {
            let value = unsafe { T::from_buffer(self.buffer, T::STRIDE * (self.len - 1)) };
            let vector = Self {
                buffer: self.buffer,
                len: self.len - 1,
                _marker: PhantomData,
            };
            Some((value, vector))
        }
    }

    /// Divides one `Vector` into two at an index.
    ///
    /// The first will contain all indices from `[0, mid)` (excluding
    /// the index `mid` itself) and the second will contain all
    /// indices from `[mid, len)` (excluding the index `len` itself).
    #[inline]
    #[must_use]
    pub fn split_at(self, mid: usize) -> Option<(Vector<'buf, T>, Vector<'buf, T>)> {
        if mid <= self.len {
            let start = Self {
                buffer: self.buffer,
                len: mid,
                _marker: PhantomData,
            };
            let tail = Self {
                buffer: self
                    .buffer
                    .advance(T::STRIDE * mid)
                    .expect("IMPOSSIBLE: the length was checked on creation"),
                len: self.len - mid,
                _marker: PhantomData,
            };
            Some((start, tail))
        } else {
            None
        }
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
        if let Some((first, remaining)) = self.0.split_first() {
            self.0 = remaining;
            Some(first)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.0.len();
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.len()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.0.last()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if let Some((_start, tail)) = self.0.split_at(n) {
            self.0 = tail;
        } else {
            self.0 = Vector::empty();
        }
        self.next()
    }
}

impl<'buf, T: VectorRead<'buf>> core::iter::DoubleEndedIterator for VectorIter<'buf, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some((last, remaining)) = self.0.split_last() {
            self.0 = remaining;
            Some(last)
        } else {
            None
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.len = self.0.len.saturating_sub(n);
        self.next_back()
    }
}

impl<'buf, T: VectorRead<'buf>> core::iter::ExactSizeIterator for VectorIter<'buf, T> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<'buf, T: VectorRead<'buf>> core::iter::FusedIterator for VectorIter<'buf, T> {}

impl<'buf, T: VectorRead<'buf> + core::fmt::Debug> core::fmt::Debug for VectorIter<'buf, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("VectorIter").field(&self.0).finish()
    }
}
