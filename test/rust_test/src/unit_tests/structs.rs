#[cfg(test)]
mod tests {
    use planus::{Builder, SliceWithStartOffset, TableRead};

    use crate::planus::structs::*;

    #[test]
    fn test_roundtrip() {
        let mut builder = Builder::new();
        let abc = Abc {
            a: -1337,
            b: true,
            c: 12345,
        };
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

        let abc_ref = table.abc().unwrap().unwrap();

        assert_eq!(abc_ref.a(), abc.a);
        assert_eq!(abc_ref.b(), abc.b);
        assert_eq!(abc_ref.c(), abc.c);
        assert_eq!(planus::ToOwned::to_owned(&abc_ref).unwrap(), abc);
    }
}
