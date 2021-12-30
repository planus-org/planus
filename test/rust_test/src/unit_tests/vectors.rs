#[cfg(test)]
mod tests {
    use planus::{Builder, SliceWithStartOffset, TableRead, ToOwned};

    use crate::planus::vectors::*;

    #[test]
    fn test_roundtrip() {
        let mut builder = Builder::new();

        let table_a = vec![
            TableA { val1: 42, val2: 43 },
            TableA { val1: 44, val2: 45 },
            TableA { val1: -1, val2: 0 },
        ];
        let empty: Vec<TableA> = vec![];

        let root = Wrap::create(&mut builder, &table_a, &empty);
        let slice = builder.finish(root, None);

        let table = WrapRef::from_buffer(
            SliceWithStartOffset {
                buffer: slice,
                offset_from_start: 0,
            },
            0,
        )
        .unwrap();

        let wrap_owned = ToOwned::to_owned(table).unwrap();
        let table_a_owned = wrap_owned.table_a.unwrap();
        let table_a_default_owned = wrap_owned.table_a_default;

        assert_eq!(table_a.len(), table_a_owned.len());
        assert!(table_a_default_owned.is_empty());
        for (t1, t2) in table_a.iter().zip(table_a_owned.iter()) {
            assert_eq!(t1.val1, t2.val1);
            assert_eq!(t1.val2, t2.val2);
        }
    }
}
