#[cfg(test)]
mod tests {
    use crate::planus::unions::*;
    use planus::{Buffer, SliceWithStartOffset, TableRead};

    #[test]
    fn test_roundtrip() {
        let mut buffer = Buffer::new();

        let table = TableA::create(&mut buffer, -32);
        let abc = Abc::create_a(&mut buffer, table);
        let root = Wrap::create(&mut buffer, &abc);
        let slice = buffer.finish(root, None);
        let table = WrapRef::from_buffer(
            SliceWithStartOffset {
                buffer: slice,
                offset_from_start: 0,
            },
            0,
        )
        .unwrap();
        let table_owned = planus::ToOwned::to_owned(&table);
        let variant = table_owned.unwrap().abc.unwrap();
        let table_inner = match variant {
            Abc::A(table_inner) => table_inner,
            _ => panic!(),
        };
        assert_eq!(table_inner.val, -32);
        buffer.clear();

        let table = TableB::create(&mut buffer, true);
        let abc = Abc::create_b(&mut buffer, table);
        let root = Wrap::create(&mut buffer, &abc);
        let slice = buffer.finish(root, None);
        let table = WrapRef::from_buffer(
            SliceWithStartOffset {
                buffer: slice,
                offset_from_start: 0,
            },
            0,
        )
        .unwrap();
        let table_owned = planus::ToOwned::to_owned(&table);
        let variant = table_owned.unwrap().abc.unwrap();
        let table_inner = match variant {
            Abc::B(table_inner) => table_inner,
            _ => panic!(),
        };
        assert_eq!(table_inner.val, true);
        buffer.clear();

        let table = TableC::create(&mut buffer, 1234567);
        let abc = Abc::create_c(&mut buffer, table);
        let root = Wrap::create(&mut buffer, &abc);
        let slice = buffer.finish(root, None);
        let table = WrapRef::from_buffer(
            SliceWithStartOffset {
                buffer: slice,
                offset_from_start: 0,
            },
            0,
        )
        .unwrap();
        let table_owned = planus::ToOwned::to_owned(&table);
        let variant = table_owned.unwrap().abc.unwrap();
        let table_inner = match variant {
            Abc::C(table_inner) => table_inner,
            _ => panic!(),
        };
        assert_eq!(table_inner.val, 1234567);
        buffer.clear();
    }
}
