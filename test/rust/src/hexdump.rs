use std::fmt::Write;

pub fn hexdump_flatbuffer_table(buf: &[u8]) -> String {
    let mut out = String::new();
    let obj_start = u32::from_le_bytes(buf[..4].try_into().unwrap()) as usize;
    let vtable_offset =
        i32::from_le_bytes(buf[obj_start..obj_start + 4].try_into().unwrap()) as isize as usize;
    let vtable_start = obj_start.wrapping_sub(vtable_offset);
    assert!(vtable_start % 2 == 0);
    let vtable_size =
        u16::from_le_bytes(buf[vtable_start..vtable_start + 2].try_into().unwrap()) as usize;
    let obj_size =
        u16::from_le_bytes(buf[vtable_start + 2..vtable_start + 4].try_into().unwrap()) as usize;
    assert!(vtable_size >= 4 && vtable_size % 2 == 0);
    assert!(obj_size >= 4);

    let vtable_end = vtable_start.checked_add(vtable_size).unwrap();
    let obj_end = obj_start.checked_add(obj_size).unwrap();

    assert!(vtable_end <= obj_start || obj_end <= vtable_start);

    writeln!(out, "obj    @ 0x{obj_start:02x}..0x{obj_end:02x}").unwrap();
    writeln!(out, "vtable @ 0x{vtable_start:02x}..0x{vtable_end:02x}").unwrap();

    let mut fields = vec![(obj_end, usize::max_value())];
    for (i, field_offset) in buf[vtable_start + 4..vtable_end]
        .chunks_exact(2)
        .enumerate()
    {
        let field_offset = u16::from_le_bytes(field_offset.try_into().unwrap()) as usize;
        if field_offset != 0 {
            fields.push((obj_start + field_offset, i));
        }
    }
    fields.sort_unstable();
    let mut fields = fields
        .windows(2)
        .map(|chunk| {
            let i = chunk[0].1;
            let start = chunk[0].0;
            let end = chunk[1].0;
            (i, start, end)
        })
        .collect::<Vec<_>>();
    fields.sort_unstable();
    for (i, start, end) in fields {
        writeln!(out, "field[{i}] @ 0x{start:02x}..0x{end:02x}:").unwrap();
        for chunk in buf[start..end].chunks(8) {
            write!(out, " ").unwrap();
            for b in chunk {
                write!(out, " {b:02x}").unwrap();
            }
            writeln!(out).unwrap();
        }
    }

    out
}
