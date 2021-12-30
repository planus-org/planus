mod backvec;
mod builder;

pub mod errors;
#[doc(hidden)]
pub mod table_reader;
#[doc(hidden)]
pub mod table_writer;

use std::{borrow::Cow, convert::TryInto, marker::PhantomData, mem::MaybeUninit};

pub use errors::Error;
use errors::ErrorKind;

pub use crate::builder::Builder;

pub type Result<T> = std::result::Result<T, Error>;
#[doc(hidden)]
pub type Cursor<'a, const N: usize> = array_init_cursor::Cursor<'a, u8, N>;

#[doc(hidden)]
pub enum Void {}

#[doc(hidden)]
pub trait Primitive {
    const ALIGNMENT: usize;
    const ALIGNMENT_MASK: usize = Self::ALIGNMENT - 1;
    const SIZE: usize;
}

pub trait ReadAsRoot<'a>: Sized {
    fn read_as_root(slice: &'a [u8]) -> Result<Self>;
}

pub trait WriteAs<P: Primitive> {
    #[doc(hidden)]
    type Prepared: WriteAsPrimitive<P>;
    #[doc(hidden)]
    fn prepare(&self, builder: &mut Builder) -> Self::Prepared;
}

pub trait WriteAsDefault<P: Primitive, D: ?Sized> {
    #[doc(hidden)]
    type Prepared: WriteAsPrimitive<P>;
    #[doc(hidden)]
    fn prepare(&self, builder: &mut Builder, default: &D) -> Option<Self::Prepared>;
}

pub trait WriteAsOptional<P: Primitive> {
    #[doc(hidden)]
    type Prepared: WriteAsPrimitive<P>;
    #[doc(hidden)]
    fn prepare(&self, builder: &mut Builder) -> Option<Self::Prepared>;
}

pub trait WriteAsOffset<T: ?Sized> {
    #[doc(hidden)]
    fn prepare(&self, builder: &mut Builder) -> Offset<T>;
}

pub trait WriteAsUnion<T: ?Sized> {
    #[doc(hidden)]
    fn prepare(&self, builder: &mut Builder) -> UnionOffset<T>;
}

pub trait WriteAsOptionalUnion<T: ?Sized> {
    #[doc(hidden)]
    fn prepare(&self, builder: &mut Builder) -> Option<UnionOffset<T>>;
}

pub trait ToOwned {
    type Value;
    fn to_owned(self) -> Result<Self::Value>;
}

#[doc(hidden)]
pub trait WriteAsPrimitive<P> {
    fn write<const N: usize>(&self, cursor: Cursor<'_, N>, buffer_position: u32);
}

#[doc(hidden)]
#[derive(Copy, Clone, Debug)]
pub struct SliceWithStartOffset<'buf> {
    pub buffer: &'buf [u8],
    pub offset_from_start: usize,
}

impl<'buf> SliceWithStartOffset<'buf> {
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_slice(&self) -> &'buf [u8] {
        self.buffer
    }

    pub fn advance(&self, amount: usize) -> std::result::Result<Self, errors::ErrorKind> {
        let buffer = self.buffer.get(amount..).ok_or(ErrorKind::InvalidOffset)?;
        Ok(Self {
            buffer,
            offset_from_start: self.offset_from_start + amount,
        })
    }

    pub fn advance_as_array<const N: usize>(
        &self,
        amount: usize,
    ) -> std::result::Result<ArrayWithStartOffset<'buf, N>, errors::ErrorKind> {
        let buffer = self
            .buffer
            .get(amount..amount + N)
            .ok_or(ErrorKind::InvalidOffset)?;
        Ok(ArrayWithStartOffset {
            buffer: buffer.try_into().unwrap(),
            offset_from_start: self.offset_from_start + amount,
        })
    }

    /// # Safety
    /// TODO
    pub unsafe fn unchecked_advance_as_array<const N: usize>(
        &self,
        amount: usize,
    ) -> ArrayWithStartOffset<'buf, N> {
        let buffer = self.buffer.get_unchecked(amount..amount + N);
        ArrayWithStartOffset {
            buffer: buffer.try_into().unwrap(),
            offset_from_start: self.offset_from_start + amount,
        }
    }
}

#[doc(hidden)]
#[derive(Copy, Clone, Debug)]
pub struct ArrayWithStartOffset<'buf, const N: usize> {
    pub buffer: &'buf [u8; N],
    pub offset_from_start: usize,
}

impl<'buf, const N: usize> ArrayWithStartOffset<'buf, N> {
    pub fn as_array(&self) -> &'buf [u8; N] {
        self.buffer
    }

    pub fn advance_as_array<const K: usize>(
        &self,
        amount: usize,
    ) -> std::result::Result<ArrayWithStartOffset<'buf, K>, errors::ErrorKind> {
        let buffer = self
            .buffer
            .get(amount..amount + K)
            .ok_or(ErrorKind::InvalidOffset)?;
        Ok(ArrayWithStartOffset {
            buffer: buffer.try_into().unwrap(),
            offset_from_start: self.offset_from_start + amount,
        })
    }
}
#[doc(hidden)]
pub trait TableRead<'buf>: Sized {
    fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
    ) -> std::result::Result<Self, ErrorKind>;
}

#[doc(hidden)]
pub trait TableReadUnion<'buf>: Sized {
    // TODO: Double-wrap the result: once for generic errors and one for unknown variants
    fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
        tag: u8,
    ) -> std::result::Result<Self, ErrorKind>;
}

impl<P: Primitive> WriteAsOptional<P> for () {
    type Prepared = Void;
    #[inline]
    fn prepare(&self, _builder: &mut Builder) -> Option<Void> {
        None
    }
}

impl<P: Primitive> WriteAsPrimitive<P> for Void {
    #[inline]
    fn write<const N: usize>(&self, _cursor: Cursor<'_, N>, _buffer_position: u32) {
        match *self {}
    }
}

impl<T: ?Sized> WriteAsOptionalUnion<T> for () {
    #[inline]
    fn prepare(&self, _builder: &mut Builder) -> Option<UnionOffset<T>> {
        None
    }
}

impl<P: Primitive, T: WriteAsOptional<P>> WriteAsOptional<P> for Option<T> {
    type Prepared = T::Prepared;
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<T::Prepared> {
        self.as_ref()?.prepare(builder)
    }
}

impl<T1, T2: WriteAsOptionalUnion<T1>> WriteAsOptionalUnion<T1> for Option<T2> {
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<UnionOffset<T1>> {
        self.as_ref()?.prepare(builder)
    }
}

pub struct Offset<T: ?Sized> {
    offset: u32,
    phantom: PhantomData<T>,
}

impl<T: ?Sized> Copy for Offset<T> {}
impl<T: ?Sized> Clone for Offset<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Offset<T> {
    #[doc(hidden)]
    pub fn downcast(&self) -> Offset<()> {
        Offset {
            offset: self.offset,
            phantom: PhantomData,
        }
    }
}

pub struct UnionOffset<T: ?Sized> {
    pub tag: u8,
    pub offset: Offset<()>,
    phantom: PhantomData<T>,
}

impl<T: ?Sized> UnionOffset<T> {
    #[doc(hidden)]
    #[inline]
    pub fn new(tag: u8, offset: Offset<()>) -> UnionOffset<T> {
        Self {
            tag,
            offset,
            phantom: PhantomData,
        }
    }
}

impl<T: ?Sized> Copy for UnionOffset<T> {}
impl<T: ?Sized> Clone for UnionOffset<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Primitive for Offset<T> {
    const ALIGNMENT: usize = 4;
    const SIZE: usize = 4;
}

impl<T: ?Sized> WriteAsPrimitive<Offset<T>> for Offset<T> {
    #[inline]
    fn write<const N: usize>(&self, cursor: Cursor<'_, N>, buffer_position: u32) {
        cursor
            .assert_size()
            .finish(u32::to_le_bytes(buffer_position - self.offset));
    }
}

impl<T: ?Sized> WriteAs<Offset<T>> for Offset<T> {
    type Prepared = Self;
    #[inline]
    fn prepare(&self, _builder: &mut Builder) -> Self {
        *self
    }
}

impl<T: ?Sized> WriteAsOptional<Offset<T>> for Offset<T> {
    type Prepared = Self;
    #[inline]
    fn prepare(&self, _builder: &mut Builder) -> Option<Self> {
        Some(*self)
    }
}

impl<T: ?Sized> WriteAsOffset<T> for Offset<T> {
    fn prepare(&self, _builder: &mut Builder) -> Offset<T> {
        *self
    }
}

impl<T: ?Sized> WriteAsUnion<T> for UnionOffset<T> {
    #[inline]
    fn prepare(&self, _builder: &mut Builder) -> Self {
        *self
    }
}

impl<T: ?Sized> WriteAsOptionalUnion<T> for UnionOffset<T> {
    #[inline]
    fn prepare(&self, _builder: &mut Builder) -> Option<Self> {
        Some(*self)
    }
}

impl<'a, P: Primitive, T: ?Sized + WriteAsPrimitive<P>> WriteAsPrimitive<P> for &'a T {
    #[inline]
    fn write<const N: usize>(&self, cursor: Cursor<'_, N>, buffer_position: u32) {
        T::write(*self, cursor, buffer_position)
    }
}

impl<'a, P: Primitive, T: ?Sized + WriteAs<P>> WriteAs<P> for &'a T {
    type Prepared = T::Prepared;
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> T::Prepared {
        T::prepare(self, builder)
    }
}

impl<'a, P: Primitive, D: ?Sized, T: ?Sized + WriteAsDefault<P, D>> WriteAsDefault<P, D> for &'a T {
    type Prepared = T::Prepared;
    #[inline]
    fn prepare(&self, builder: &mut Builder, default: &D) -> Option<T::Prepared> {
        T::prepare(self, builder, default)
    }
}

impl<'a, P: Primitive, T: ?Sized + WriteAsOptional<P>> WriteAsOptional<P> for &'a T {
    type Prepared = T::Prepared;
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<T::Prepared> {
        T::prepare(self, builder)
    }
}

impl<'a, T1: ?Sized, T2: ?Sized + WriteAsOffset<T1>> WriteAsOffset<T1> for &'a T2 {
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Offset<T1> {
        T2::prepare(self, builder)
    }
}

impl<'a, T1: ?Sized, T2: ?Sized + WriteAsUnion<T1>> WriteAsUnion<T1> for &'a T2 {
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> UnionOffset<T1> {
        T2::prepare(self, builder)
    }
}

impl<'a, T1: ?Sized, T2: ?Sized + WriteAsOptionalUnion<T1>> WriteAsOptionalUnion<T1> for &'a T2 {
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<UnionOffset<T1>> {
        T2::prepare(self, builder)
    }
}

impl<'a, T: ?Sized + ToOwned + Copy> ToOwned for &'a T {
    type Value = T::Value;

    #[inline]
    fn to_owned(self) -> Result<Self::Value> {
        T::to_owned(*self)
    }
}

impl<'a, P: Primitive, T: ?Sized + VectorWrite<P>> VectorWrite<P> for &'a T {
    const STRIDE: usize = T::STRIDE;
    type Value = T::Value;

    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Self::Value {
        T::prepare(self, builder)
    }

    #[inline]
    unsafe fn write_values(
        values: &[Self::Value],
        bytes: *mut MaybeUninit<u8>,
        buffer_position: u32,
    ) {
        T::write_values(values, bytes, buffer_position);
    }
}

impl<P: Primitive, T: ?Sized + WriteAs<P>> WriteAs<P> for Box<T> {
    type Prepared = T::Prepared;
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> T::Prepared {
        T::prepare(self, builder)
    }
}

impl<P: Primitive, D: ?Sized, T: ?Sized + WriteAsDefault<P, D>> WriteAsDefault<P, D> for Box<T> {
    type Prepared = T::Prepared;
    #[inline]
    fn prepare(&self, builder: &mut Builder, default: &D) -> Option<T::Prepared> {
        T::prepare(self, builder, default)
    }
}

impl<P: Primitive, T: ?Sized + WriteAsOptional<P>> WriteAsOptional<P> for Box<T> {
    type Prepared = T::Prepared;
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<T::Prepared> {
        T::prepare(self, builder)
    }
}

impl<P, T: ?Sized + WriteAsOffset<P>> WriteAsOffset<P> for Box<T> {
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Offset<P> {
        T::prepare(self, builder)
    }
}

impl<T1: ?Sized, T2: ?Sized + WriteAsUnion<T1>> WriteAsUnion<T1> for Box<T2> {
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> UnionOffset<T1> {
        T2::prepare(self, builder)
    }
}

impl<T1: ?Sized, T2: ?Sized + WriteAsOptionalUnion<T1>> WriteAsOptionalUnion<T1> for Box<T2> {
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<UnionOffset<T1>> {
        T2::prepare(self, builder)
    }
}

macro_rules! gen_primitive_types {
    ($ty:ty, $size:expr) => {
        impl Primitive for $ty {
            const ALIGNMENT: usize = $size;
            const SIZE: usize = $size;
        }

        impl WriteAsPrimitive<$ty> for $ty {
            #[inline]
            fn write<const N: usize>(&self, cursor: Cursor<'_, N>, _buffer_position: u32) {
                cursor.assert_size().finish(self.to_le_bytes());
            }
        }

        impl WriteAs<$ty> for $ty {
            type Prepared = Self;
            #[inline]
            fn prepare(&self, _builder: &mut Builder) -> Self {
                *self
            }
        }

        impl WriteAsDefault<$ty, $ty> for $ty {
            type Prepared = Self;
            #[inline]
            fn prepare(&self, _builder: &mut Builder, default: &$ty) -> Option<Self> {
                #[allow(clippy::float_cmp)]
                if self == default {
                    None
                } else {
                    Some(*self)
                }
            }
        }

        impl WriteAsOptional<$ty> for $ty {
            type Prepared = Self;
            #[inline]
            fn prepare(&self, _builder: &mut Builder) -> Option<Self> {
                Some(*self)
            }
        }

        impl ToOwned for $ty {
            type Value = $ty;

            #[inline]
            fn to_owned(self) -> Result<$ty> {
                Ok(self)
            }
        }

        impl<'buf> TableRead<'buf> for $ty {
            #[inline]
            fn from_buffer(
                buffer: SliceWithStartOffset<'buf>,
                offset: usize,
            ) -> std::result::Result<$ty, ErrorKind> {
                let buffer = buffer.advance_as_array(offset)?.as_array();
                Ok(<$ty>::from_le_bytes(*buffer))
            }
        }

        impl<'buf> VectorRead<'buf> for $ty {
            type Output = $ty;

            #[doc(hidden)]
            const STRIDE: usize = $size;
            #[doc(hidden)]
            #[inline]
            unsafe fn from_buffer(
                buffer: SliceWithStartOffset<'buf>,
                offset: usize,
            ) -> Self::Output {
                let buffer = buffer.unchecked_advance_as_array(offset).as_array();
                <$ty>::from_le_bytes(*buffer)
            }
        }

        impl VectorWrite<$ty> for $ty {
            const STRIDE: usize = $size;
            type Value = $ty;
            #[inline]
            fn prepare(&self, _builder: &mut Builder) -> Self::Value {
                *self
            }

            #[inline]
            unsafe fn write_values(
                values: &[$ty],
                bytes: *mut MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [MaybeUninit<u8>; $size];
                for (i, v) in values.iter().enumerate() {
                    v.write(
                        Cursor::new(&mut *bytes.add(i)),
                        buffer_position - ($size * i) as u32,
                    );
                }
            }
        }
    };
}

gen_primitive_types!(i8, 1);
gen_primitive_types!(u8, 1);
gen_primitive_types!(i16, 2);
gen_primitive_types!(u16, 2);
gen_primitive_types!(i32, 4);
gen_primitive_types!(u32, 4);
gen_primitive_types!(i64, 8);
gen_primitive_types!(u64, 8);
gen_primitive_types!(f32, 4);
gen_primitive_types!(f64, 8);

impl Primitive for bool {
    const ALIGNMENT: usize = 1;
    const SIZE: usize = 1;
}

impl WriteAsPrimitive<bool> for bool {
    #[inline]
    fn write<const N: usize>(&self, cursor: Cursor<'_, N>, _buffer_position: u32) {
        cursor.assert_size().finish([if *self { 1 } else { 0 }]);
    }
}

impl WriteAs<bool> for bool {
    type Prepared = Self;
    #[inline]
    fn prepare(&self, _builder: &mut Builder) -> Self {
        *self
    }
}

impl WriteAsDefault<bool, bool> for bool {
    type Prepared = Self;
    #[inline]
    fn prepare(&self, _builder: &mut Builder, default: &bool) -> Option<bool> {
        if self == default {
            None
        } else {
            Some(*self)
        }
    }
}

impl WriteAsOptional<bool> for bool {
    type Prepared = Self;
    #[inline]
    fn prepare(&self, _builder: &mut Builder) -> Option<Self> {
        Some(*self)
    }
}

impl ToOwned for bool {
    type Value = bool;

    #[inline]
    fn to_owned(self) -> Result<bool> {
        Ok(self)
    }
}

impl<T: ToOwned, E> ToOwned for std::result::Result<T, E>
where
    errors::Error: From<E>,
{
    type Value = T::Value;

    #[inline]
    fn to_owned(self) -> Result<Self::Value> {
        self?.to_owned()
    }
}

impl<'buf> TableRead<'buf> for bool {
    #[inline]
    fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
    ) -> std::result::Result<bool, ErrorKind> {
        Ok(buffer.advance_as_array::<1>(offset)?.as_array()[0] != 0)
    }
}

impl<'buf> VectorRead<'buf> for bool {
    type Output = bool;
    const STRIDE: usize = 1;

    #[inline]
    unsafe fn from_buffer(buffer: SliceWithStartOffset<'buf>, offset: usize) -> bool {
        *buffer.as_slice().get_unchecked(offset) != 0
    }
}

impl VectorWrite<bool> for bool {
    const STRIDE: usize = 1;

    type Value = bool;

    #[inline]
    fn prepare(&self, _builder: &mut Builder) -> Self::Value {
        *self
    }

    #[inline]
    unsafe fn write_values(
        values: &[Self::Value],
        bytes: *mut MaybeUninit<u8>,
        buffer_position: u32,
    ) {
        let bytes = bytes as *mut [MaybeUninit<u8>; 1];
        for (i, v) in values.iter().enumerate() {
            v.write(Cursor::new(&mut *bytes.add(i)), buffer_position - i as u32);
        }
    }
}

pub trait VectorRead<'buf> {
    type Output;

    #[doc(hidden)]
    const STRIDE: usize;
    #[doc(hidden)]
    unsafe fn from_buffer(buffer: SliceWithStartOffset<'buf>, offset: usize) -> Self::Output;
}

pub struct Vector<'buf, T: ?Sized> {
    buffer: SliceWithStartOffset<'buf>,
    len: usize,
    _marker: PhantomData<&'buf T>,
}

impl<'buf, T: ?Sized> Copy for Vector<'buf, T> {}
impl<'buf, T: ?Sized> Clone for Vector<'buf, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
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
        VectorIter {
            buffer: self.buffer,
            len: self.len,
            _marker: PhantomData,
        }
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

pub struct VectorIter<'buf, T: ?Sized> {
    buffer: SliceWithStartOffset<'buf>,
    len: usize,
    _marker: PhantomData<&'buf T>,
}

impl<'buf, T: ?Sized + VectorRead<'buf>> Iterator for VectorIter<'buf, T> {
    type Item = T::Output;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            let result = unsafe { T::from_buffer(self.buffer, 0) };
            self.buffer = self
                .buffer
                .advance(T::STRIDE)
                .expect("IMPOSSIBLE: we checked the length on creation");
            self.len -= 1;
            Some(result)
        } else {
            None
        }
    }
}

fn array_from_buffer(
    buffer: SliceWithStartOffset<'_>,
    offset: usize,
) -> std::result::Result<(SliceWithStartOffset<'_>, usize), ErrorKind> {
    let value: u32 = TableRead::from_buffer(buffer, offset)?;
    let vector_offset = offset
        .checked_add(value as usize)
        .ok_or(ErrorKind::InvalidOffset)?;
    let buffer = buffer.advance(vector_offset)?;
    let len: u32 = TableRead::from_buffer(buffer, 0)?;
    Ok((buffer.advance(4)?, len as usize))
}

impl<'buf, T: ?Sized + VectorRead<'buf>> TableRead<'buf> for Vector<'buf, T> {
    fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
    ) -> std::result::Result<Self, ErrorKind> {
        let (buffer, len) = array_from_buffer(buffer, offset)?;
        if len.checked_mul(T::STRIDE).ok_or(ErrorKind::InvalidLength)? <= buffer.len() {
            Ok(Vector {
                buffer,
                len,
                _marker: PhantomData,
            })
        } else {
            Err(ErrorKind::InvalidLength)
        }
    }
}

impl<'buf, T: ?Sized + VectorRead<'buf>> ToOwned for Vector<'buf, T>
where
    T::Output: ToOwned,
{
    type Value = Vec<<T::Output as ToOwned>::Value>;

    fn to_owned(self) -> std::result::Result<Self::Value, Error> {
        self.iter().map(|v| v.to_owned()).collect()
    }
}

impl<'buf, T: ?Sized + VectorRead<'buf>> std::fmt::Debug for Vector<'buf, T>
where
    T::Output: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

pub trait VectorWrite<P> {
    #[doc(hidden)]
    const STRIDE: usize;
    #[doc(hidden)]
    type Value: WriteAsPrimitive<P> + Sized;
    #[doc(hidden)]
    fn prepare(&self, builder: &mut Builder) -> Self::Value;
    #[doc(hidden)]
    unsafe fn write_values(
        values: &[Self::Value],
        bytes: *mut MaybeUninit<u8>,
        buffer_position: u32,
    );
}

impl<T: ?Sized> VectorWrite<Offset<T>> for Offset<T> {
    const STRIDE: usize = 4;
    type Value = Offset<T>;

    #[inline]
    fn prepare(&self, _builder: &mut Builder) -> Self::Value {
        *self
    }

    #[inline]
    unsafe fn write_values(
        values: &[Offset<T>],
        bytes: *mut MaybeUninit<u8>,
        buffer_position: u32,
    ) {
        let bytes = bytes as *mut [MaybeUninit<u8>; 4];
        for (i, v) in values.iter().enumerate() {
            v.write(
                Cursor::new(&mut *bytes.add(i)),
                buffer_position - (Self::STRIDE * i) as u32,
            );
        }
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

impl<T, P: Primitive> WriteAsOffset<[P]> for [T]
where
    T: VectorWrite<P>,
{
    fn prepare(&self, builder: &mut Builder) -> Offset<[P]> {
        let mut tmp: Vec<T::Value> = Vec::with_capacity(self.len());
        for v in self.iter() {
            tmp.push(v.prepare(builder));
        }
        unsafe {
            builder.write_with(
                4 + T::STRIDE.checked_mul(self.len()).unwrap(),
                P::ALIGNMENT_MASK.max(3),
                |buffer_position, bytes| {
                    let bytes = bytes.as_mut_ptr();

                    (self.len() as u32).write(
                        Cursor::new(&mut *(bytes as *mut [MaybeUninit<u8>; 4])),
                        buffer_position,
                    );

                    T::write_values(&tmp, bytes.add(4), buffer_position - 4);
                },
            )
        };
        builder.current_offset()
    }
}

impl<T, P, const N: usize> WriteAs<Offset<[P]>> for [T; N]
where
    P: Primitive,
    T: VectorWrite<P>,
{
    type Prepared = Offset<[P]>;

    fn prepare(&self, builder: &mut Builder) -> Offset<[P]> {
        WriteAsOffset::prepare(self, builder)
    }
}

impl<T, P, const N: usize> WriteAsOptional<Offset<[P]>> for [T; N]
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

impl<T, P, const N: usize> WriteAsOffset<[P]> for [T; N]
where
    P: Primitive,
    T: VectorWrite<P>,
{
    fn prepare(&self, builder: &mut Builder) -> Offset<[P]> {
        let mut tmp: [MaybeUninit<T::Value>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for (t, v) in tmp.iter_mut().zip(self.iter()) {
            t.write(v.prepare(builder));
        }
        // TODO: replace with std::mem::MaybeUninit::array_assume_init when it becomes stable
        //       https://github.com/rust-lang/rust/issues/80908
        let tmp =
            unsafe { (&tmp as *const [MaybeUninit<T::Value>; N] as *const [T::Value; N]).read() };
        unsafe {
            builder.write_with(
                4 + T::STRIDE.checked_mul(self.len()).unwrap(),
                P::ALIGNMENT_MASK.max(3),
                |buffer_position, bytes| {
                    let bytes = bytes.as_mut_ptr();

                    (self.len() as u32).write(
                        Cursor::new(&mut *(bytes as *mut [MaybeUninit<u8>; 4])),
                        buffer_position,
                    );

                    T::write_values(&tmp, bytes.add(4), buffer_position - 4);
                },
            )
        };
        builder.current_offset()
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

impl WriteAs<Offset<str>> for str {
    type Prepared = Offset<str>;

    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Offset<str> {
        WriteAsOffset::prepare(self, builder)
    }
}

impl WriteAsDefault<Offset<str>, str> for str {
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

impl WriteAsOptional<Offset<str>> for str {
    type Prepared = Offset<str>;
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<Offset<str>> {
        Some(WriteAsOffset::prepare(self, builder))
    }
}

impl WriteAsOffset<str> for str {
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Offset<str> {
        let offset = <[u8] as WriteAsOffset<[u8]>>::prepare(self.as_bytes(), builder);
        Offset {
            offset: offset.offset,
            phantom: PhantomData,
        }
    }
}

impl<'a> ToOwned for &'a str {
    type Value = String;

    fn to_owned(self) -> Result<Self::Value> {
        Ok(self.to_string())
    }
}

impl<'buf> VectorRead<'buf> for str {
    type Output = Result<&'buf str>;

    #[doc(hidden)]
    const STRIDE: usize = 4;
    #[doc(hidden)]
    #[inline]
    unsafe fn from_buffer(buffer: SliceWithStartOffset<'buf>, offset: usize) -> Self::Output {
        let add_context =
            |e: ErrorKind| e.with_error_location("[str]", "get", buffer.offset_from_start);
        let (slice, len) = array_from_buffer(buffer, offset).map_err(add_context)?;
        let slice = slice
            .as_slice()
            .get(..len)
            .ok_or(ErrorKind::InvalidLength)
            .map_err(add_context)?;
        let str = std::str::from_utf8(slice)
            .map_err(|source| ErrorKind::InvalidUtf8 { source })
            .map_err(add_context)?;
        Ok(str)
    }
}

impl VectorWrite<Offset<str>> for str {
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

impl<'buf> TableRead<'buf> for &'buf str {
    fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
    ) -> std::result::Result<Self, ErrorKind> {
        let slice: &[u8] = TableRead::from_buffer(buffer, offset)?;
        Ok(std::str::from_utf8(slice)?)
    }
}

impl WriteAs<Offset<str>> for String {
    type Prepared = Offset<str>;

    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Offset<str> {
        WriteAsOffset::prepare(self.as_str(), builder)
    }
}

impl WriteAsDefault<Offset<str>, str> for String {
    type Prepared = Offset<str>;

    #[inline]
    fn prepare(&self, builder: &mut Builder, default: &str) -> Option<Offset<str>> {
        if self == default {
            None
        } else {
            Some(WriteAsOffset::prepare(self.as_str(), builder))
        }
    }
}

impl WriteAsOptional<Offset<str>> for String {
    type Prepared = Offset<str>;

    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Option<Offset<str>> {
        Some(WriteAsOffset::prepare(self.as_str(), builder))
    }
}

impl WriteAsOffset<str> for String {
    #[inline]
    fn prepare(&self, builder: &mut Builder) -> Offset<str> {
        WriteAsOffset::prepare(self.as_str(), builder)
    }
}

impl<'a> ToOwned for String {
    type Value = String;

    fn to_owned(self) -> Result<Self::Value> {
        Ok(self)
    }
}

impl VectorWrite<Offset<str>> for String {
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

impl<'buf> TableRead<'buf> for &'buf [u8] {
    fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
    ) -> std::result::Result<Self, ErrorKind> {
        let (buffer, len) = array_from_buffer(buffer, offset)?;
        buffer.as_slice().get(..len).ok_or(ErrorKind::InvalidLength)
    }
}

impl<'buf> TableRead<'buf> for Cow<'buf, str> {
    fn from_buffer(
        buffer: SliceWithStartOffset<'buf>,
        offset: usize,
    ) -> std::result::Result<Self, ErrorKind> {
        let bytes = <&'buf [u8] as TableRead<'buf>>::from_buffer(buffer, offset)?;
        Ok(String::from_utf8_lossy(bytes))
    }
}
