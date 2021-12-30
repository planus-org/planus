#[cfg(test)]
mod tests {
    use planus::{Builder, SliceWithStartOffset, TableRead};

    use crate::planus::enums::*;

    #[test]
    fn test_roundtrip() {
        let mut builder = Builder::new();

        for var in [Abc::A, Abc::B, Abc::C] {
            let root = Wrap::create(&mut builder, var);
            let slice = builder.finish(root, None);

            let table = WrapRef::from_buffer(
                SliceWithStartOffset {
                    buffer: slice,
                    offset_from_start: 0,
                },
                0,
            )
            .unwrap();
            println!("{:?}", table);

            assert_eq!(table.abc().unwrap(), var);
            builder.clear();
        }
    }
}
