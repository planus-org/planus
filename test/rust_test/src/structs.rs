#[cfg(test)]
mod tests {
    use planus::{Buffer, SliceWithStartOffset, TableRead};

    use crate::planus::structs::*;

    #[test]
    fn test_roundtrip() {
        let mut buffer = Buffer::new();
        let abc = Abc {
            a: -1337,
            b: true,
            c: 12345,
        };
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

        let abc_ref = table.abc().unwrap().unwrap();

        assert_eq!(abc_ref.a(), abc.a);
        assert_eq!(abc_ref.b(), abc.b);
        assert_eq!(abc_ref.c(), abc.c);
        assert_eq!(planus::ToOwned::to_owned(&abc_ref).unwrap(), abc);
    }
}
