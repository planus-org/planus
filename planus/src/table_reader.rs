use crate::{errors::ErrorKind, BufferWithStartOffset, TableRead, TableReadUnion};
use std::convert::TryInto;

#[derive(Copy, Clone, Debug)]
pub struct Table<'buf> {
    object: BufferWithStartOffset<'buf>,
    vtable: &'buf [u8],
}

impl<'buf> Table<'buf> {
    pub fn from_buffer(
        buffer: BufferWithStartOffset<'buf>,
        field_offset: usize,
    ) -> Result<Self, ErrorKind> {
        let field_value = u32::from_buffer(buffer, field_offset)?;
        let object_offset = field_offset
            .checked_add(field_value as usize)
            .ok_or(ErrorKind::InvalidOffset)?;
        let object = buffer.advance(object_offset)?;

        let vtable_offset_relative = i32::from_buffer(buffer, object_offset)?;
        let vtable_offset: usize = (object_offset as i64)
            .checked_sub(vtable_offset_relative as i64)
            .ok_or(ErrorKind::InvalidOffset)?
            .try_into()
            .map_err(|_| ErrorKind::InvalidOffset)?;

        let vtable_size = u16::from_buffer(buffer, vtable_offset)?;
        if vtable_size < 4 || vtable_size % 2 != 0 {
            return Err(ErrorKind::InvalidVtableLength {
                length: vtable_size,
            });
        }
        let vtable_full = buffer
            .advance(vtable_offset)?
            .as_slice()
            .get(..vtable_size as usize)
            .ok_or(ErrorKind::InvalidLength)?;
        let vtable = vtable_full.get(4..).ok_or(ErrorKind::InvalidOffset)?;

        Ok(Self { object, vtable })
    }

    pub fn access<T: TableRead<'buf>>(&self, vtable_offset: usize) -> Result<Option<T>, ErrorKind> {
        let offset = self
            .vtable
            .get(2 * vtable_offset..2 * (vtable_offset + 1))
            .expect("IMPOSSIBLE: trying to access invalid vtable offset");
        let offset = u16::from_le_bytes(offset.try_into().unwrap()) as usize;
        if offset != 0 {
            T::from_buffer(self.object, offset).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn access_union<T: TableReadUnion<'buf>>(
        &self,
        vtable_offset: usize,
    ) -> Result<Option<T>, ErrorKind> {
        let offset = self
            .vtable
            .get(2 * vtable_offset..2 * (vtable_offset + 2))
            .expect("IMPOSSIBLE: trying to access invalid vtable offset for union");
        let tag_offset = u16::from_le_bytes(offset[..2].try_into().unwrap()) as usize;
        let value_offset = u16::from_le_bytes(offset[2..].try_into().unwrap()) as usize;
        let tag = u8::from_buffer(self.object, tag_offset)?;
        if tag_offset != 0 && value_offset != 0 {
            T::from_buffer(self.object, value_offset, tag).map(Some)
        } else {
            Ok(None)
        }
    }
}
