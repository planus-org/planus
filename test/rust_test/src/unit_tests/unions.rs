#[cfg(test)]
mod tests {
    use planus::{Builder, SliceWithStartOffset, TableRead};

    use crate::planus::unions::*;

    #[test]
    fn test_roundtrip() {
        let mut builder = Builder::new();

        let table = TableA::create(&mut builder, -32);
        let abc = Abc::create_a(&mut builder, table);
        let root = Wrap::create(&mut builder, &abc);
        let slice = builder.finish(root, None);
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
        builder.clear();

        let table = TableB::create(&mut builder, true);
        let abc = Abc::create_b(&mut builder, table);
        let root = Wrap::create(&mut builder, &abc);
        let slice = builder.finish(root, None);
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
        assert!(table_inner.val);
        builder.clear();

        let table = TableC::create(&mut builder, 1234567);
        let abc = Abc::create_c(&mut builder, table);
        let root = Wrap::create(&mut builder, &abc);
        let slice = builder.finish(root, None);
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
        builder.clear();
    }
}
