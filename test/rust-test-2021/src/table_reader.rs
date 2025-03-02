use planus::{SliceWithStartOffset, TableReadUnion};

#[derive(Debug, PartialEq, Eq)]
struct A {
    tag: u8,
    offset: usize,
}

impl<'buf> TableReadUnion<'buf> for A {
    fn from_buffer(
        _: SliceWithStartOffset<'buf>,
        tag: u8,
        offset: usize,
    ) -> core::result::Result<Self, planus::errors::ErrorKind> {
        Ok(Self { tag, offset })
    }
}

#[test]
fn access_union() {
    use planus::table_reader::Table;
    #[rustfmt::skip]
    let data = alloc::vec![
        // object offset (u32)
        12, 0, 0, 0,
        // vtable size (u16) and object size (u16)
        8, 0, 0, 0,
        // vtable entries for tag (u16) and value (u16)
        4, 0, 99, 0,
        // the object starts with an offset for the vtable (i32)
        8, 0, 0, 0,
        // the actual tag value (u8)
        42, 0, 0, 0,
    ];
    let data = SliceWithStartOffset {
        buffer: &data,
        offset_from_start: 0,
    };
    let table = Table::from_buffer(data, 0).unwrap();

    // vtable has 1 entry
    // => accessing index 0 must be ok
    assert_eq!(
        table.access_union::<A>(0, "", "").unwrap(),
        Some(A {
            tag: 42,
            offset: 99
        })
    );
    //  => accessing index 1 must error
    assert!(table.access_union::<A>(1, "", "").is_err())
}
