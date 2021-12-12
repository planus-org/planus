#[cfg(test)]
mod tests {
    use crate::planus::vectors::*;
    use planus::{Buffer, SliceWithStartOffset, TableRead, ToOwned};

    #[test]
    fn test_roundtrip() {
        let mut buffer = Buffer::new();

        let table_a = vec![
            TableA { val1: 42, val2: 43 },
            TableA { val1: 44, val2: 45 },
            TableA { val1: -1, val2: 0 },
        ];

        let root = Wrap::create(&mut buffer, &table_a);
        let slice = buffer.finish(root, None);

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

        assert_eq!(table_a.len(), table_a_owned.len());
        for (t1, t2) in table_a.iter().zip(table_a_owned.iter()) {
            assert_eq!(t1.val1, t2.val1);
            assert_eq!(t1.val2, t2.val2);
        }
    }
}
