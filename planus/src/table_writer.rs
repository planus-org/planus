use core::mem::MaybeUninit;

use crate::{Builder, Offset, Primitive, WriteAsPrimitive};

pub struct TableWriter<'buf, const VTABLE_MAX_BYTES: usize, const OBJECT_MAX_BYTES: usize> {
    builder: &'buf mut Builder,
    vtable: [u8; VTABLE_MAX_BYTES],
    object: [u8; OBJECT_MAX_BYTES],
    vtable_size: usize,
    object_size: usize,
    object_offset: u32,
    object_alignment_mask: usize,
}

impl<'buf, const VTABLE_MAX_BYTES: usize, const OBJECT_MAX_BYTES: usize>
    TableWriter<'buf, VTABLE_MAX_BYTES, OBJECT_MAX_BYTES>
{
    #[inline]
    pub fn new(builder: &'buf mut Builder) -> Self {
        Self {
            builder,
            vtable: [0; VTABLE_MAX_BYTES], // including vtable size and object size
            object: [0; OBJECT_MAX_BYTES], // including vtable offset
            vtable_size: 4,                // for vtable size and object size
            object_size: 4,                // for offset to vtable
            object_offset: 0,              // set to the offset of the object once we know it
            object_alignment_mask: i32::ALIGNMENT_MASK, // objects always contain a vtable offset, which is an i32 aligned
        }
    }

    /// # Safety
    ///
    /// Must be called in alignment order with the most-aligned object first
    #[inline]
    pub fn calculate_size<P: Primitive>(&mut self, vtable_index: usize) {
        self.object_alignment_mask = self.object_alignment_mask.max(P::ALIGNMENT_MASK);

        self.vtable[vtable_index_to_offset(vtable_index)..][..2]
            .copy_from_slice(&(self.object_size as u16).to_le_bytes());

        self.object_size += P::SIZE;

        self.vtable_size = self.vtable_size.max(2 * (vtable_index + 1) + 4);
        debug_assert!(self.vtable_size <= self.vtable.len());
    }

    #[inline]
    pub fn finish_calculating(&mut self) {
        self.vtable[0..2].copy_from_slice(&(self.vtable_size as u16).to_le_bytes());
        self.vtable[2..4].copy_from_slice(&(self.object_size as u16).to_le_bytes());

        let vtable_offset = self.builder.write_vtable(&self.vtable[..self.vtable_size]);
        self.object_offset =
            self.builder
                .prepare_write(self.object_size - 4, self.object_alignment_mask) as u32
                + 4;
        self.object[..4]
            .copy_from_slice(&(vtable_offset as i32 - self.object_offset as i32).to_le_bytes());
        // Reset to be able to re-use in the write function
        self.object_size = 4;
    }

    /// # Safety
    ///
    /// Must be called in alignment order with the most-aligned object first
    #[inline(always)]
    pub unsafe fn write<P: Primitive, T: WriteAsPrimitive<P>, const SIZE: usize>(
        &mut self,
        value: &T,
    ) {
        assert_eq!(P::SIZE, SIZE);
        debug_assert!(self.object_size + P::SIZE <= OBJECT_MAX_BYTES);
        let slice = self.object.as_mut_ptr().cast::<u8>().add(self.object_size);
        value.write(
            array_init_cursor::Cursor::new(&mut *(slice as *mut [MaybeUninit<u8>; SIZE])),
            self.object_offset - self.object_size as u32,
        );
        self.object_size += P::SIZE;
    }

    pub fn finish<T>(self) -> Offset<T> {
        self.builder.write(&self.object[4..self.object_size]);
        self.builder.prepare_write(4, i32::ALIGNMENT_MASK);
        self.builder.write(&self.object[..4]);
        let offset = self.builder.current_offset();
        debug_assert_eq!(offset.offset, self.object_offset);
        offset
    }
}

fn vtable_index_to_offset(vtable_index: usize) -> usize {
    2 * (vtable_index + 2) // 2 bytes per index, skip the vtable size and object size
}
