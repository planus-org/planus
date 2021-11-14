use crate::{Buffer, Offset, Primitive, WriteAsPrimitive};

pub struct TableWriter<'buf, const VTABLE_MAX_BYTES: usize, const OBJECT_MAX_BYTES: usize> {
    buffer: &'buf mut Buffer,
    vtable: [u8; VTABLE_MAX_BYTES],
    vtable_size: usize,
    object: [u8; OBJECT_MAX_BYTES],
    buffer_position: usize,
    object_size: usize,
    object_alignment_mask: usize,
    position: usize,
}

impl<'buf, const VTABLE_MAX_BYTES: usize, const OBJECT_MAX_BYTES: usize>
    TableWriter<'buf, VTABLE_MAX_BYTES, OBJECT_MAX_BYTES>
{
    pub fn new(buffer: &'buf mut Buffer) -> Self {
        Self {
            buffer,
            vtable: [0; VTABLE_MAX_BYTES], // not including vtable size and object size
            object: [0; OBJECT_MAX_BYTES], // not including vtable offset
            buffer_position: 0,
            vtable_size: 0, // for vtable size and object size
            object_size: 0, // for offset to vtable
            object_alignment_mask: i32::ALIGNMENT_MASK, // objects must always contain a vtable offset
            position: 4,                                // start after offset to vtable
        }
    }

    /// Must be called in the field order
    pub fn calculate_size<P: Primitive>(&mut self, vtable_offset_end: usize) {
        self.object_alignment_mask = self.object_alignment_mask.max(P::ALIGNMENT_MASK);
        self.object_size += P::SIZE;

        debug_assert!(vtable_offset_end <= self.vtable.len() + 4);
        self.vtable_size = vtable_offset_end;
    }

    pub fn finish_calculating(&mut self) {
        self.buffer_position = self.buffer.get_buffer_position_and_prepare_write(
            self.vtable_size,
            self.object_size,
            self.object_alignment_mask,
        );
    }

    /// Must be called with the most-aligned object first
    #[inline(always)]
    pub unsafe fn write<P: Primitive, T: WriteAsPrimitive<P>>(
        &mut self,
        vtable_index: usize,
        value: &T,
    ) {
        debug_assert!(self.position + P::SIZE <= OBJECT_MAX_BYTES + 4);
        self.vtable[vtable_index_to_offset(vtable_index)..][..2]
            .copy_from_slice(&(self.position as u16).to_le_bytes());
        let slice = self.object.as_mut_ptr().cast::<u8>().add(self.position - 4);
        value.write(slice, (self.buffer_position - self.position) as u32);
        self.position += P::SIZE;
    }

    pub fn finish<T>(self) -> Offset<T> {
        self.buffer.write(&self.vtable[..self.vtable_size]);
        self.buffer
            .write(&((self.object_size + 4) as u16).to_le_bytes());
        self.buffer
            .write(&((self.vtable_size + 4) as u16).to_le_bytes());
        let offset = self.buffer.current_offset::<()>().offset;
        self.buffer
            .prepare_write(self.object_size, self.object_alignment_mask);
        self.buffer.write(&self.object[..self.object_size]);
        self.buffer.prepare_write(4, i32::ALIGNMENT_MASK);
        self.buffer
            .write(&(offset as i32 - self.buffer_position as i32).to_le_bytes());
        self.buffer.current_offset()
    }
}

fn vtable_index_to_offset(vtable_index: usize) -> usize {
    2 * vtable_index // 2 bytes per index, skip the vtable size and object size
}
