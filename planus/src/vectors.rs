use crate::{slice_helpers::SliceWithStartOffset, traits::VectorRead};
use core::marker::PhantomData;

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

impl<'buf, T: ?Sized + VectorRead<'buf>> core::fmt::Debug for Vector<'buf, T>
where
    T::Output: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: ?Sized + 'static> Vector<'static, T> {
    pub const EMPTY: Self = Self {
        buffer: SliceWithStartOffset {
            buffer: &[],
            offset_from_start: 0,
        },
        len: 0,
        _marker: PhantomData,
    };
}

impl<'buf, T: ?Sized + VectorRead<'buf>> Vector<'buf, T> {
    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    pub fn len(self) -> usize {
        self.len
    }

    #[inline]
    pub fn get(self, index: usize) -> Option<T::Output> {
        if index < self.len {
            Some(unsafe { T::from_buffer(self.buffer, T::STRIDE * index) })
        } else {
            None
        }
    }

    #[inline]
    pub fn iter(self) -> VectorIter<'buf, T> {
        VectorIter(self)
    }
}

impl<'buf, T: ?Sized + VectorRead<'buf>> IntoIterator for Vector<'buf, T> {
    type Item = T::Output;
    type IntoIter = VectorIter<'buf, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone)]
pub struct VectorIter<'buf, T: ?Sized>(Vector<'buf, T>);

impl<'buf, T: ?Sized + VectorRead<'buf>> Iterator for VectorIter<'buf, T> {
    type Item = T::Output;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
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

impl<'buf, T: ?Sized + VectorRead<'buf>> core::fmt::Debug for VectorIter<'buf, T>
where
    T::Output: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("VectorIter").field(&self.0).finish()
    }
}
