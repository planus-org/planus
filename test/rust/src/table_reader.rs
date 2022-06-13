use planus::{SliceWithStartOffset, TableReadUnion};

#[derive(Debug, PartialEq, Eq)]
struct A {
    tag: u8,
    offset: usize,
}

impl<'buf> TableReadUnion<'buf> for A {
    fn from_buffer(
        _: SliceWithStartOffset<'buf>,
        offset: usize,
        tag: u8,
    ) -> core::result::Result<Self, planus::errors::ErrorKind> {
        Ok(Self { tag, offset })
    }
}

#[test]
fn access_union() {
    use planus::table_reader::Table;
    let data: Vec<u8> = vec![
        8, 0, 0, 0, // vtable size and object size
        4, 0, 99, 0, // vtable with tag offset (u16) and value offset (u16)
        4, 0, 0, 0, // object offset (u32)
        12, 0, 0, 0, // vtable offset (i32)
        42, 0, 0, 0, // tag
    ];
    let data = SliceWithStartOffset {
        buffer: &data,
        offset_from_start: 0,
    };
    let table = Table::from_buffer(data, 8).unwrap();

    // vtable has 1 entry
    // => accessing index 0 must be ok
    assert_eq!(
        table.access_union::<A>(0, "", "").unwrap(),
        Some(A { tag: 42, offset: 99 })
    );
    //  => accessing index 1 must error
    assert!(table.access_union::<A>(1, "", "").is_err())
}
