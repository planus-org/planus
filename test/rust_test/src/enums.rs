#[cfg(test)]
mod tests {
    use crate::planus::enums::*;
    use planus::{Buffer, SliceWithStartOffset, TableRead};

    #[test]
    fn test_roundtrip() {
        let mut buffer = Buffer::new();

        for var in [Abc::A, Abc::B, Abc::C] {
            let root = Wrap::create(&mut buffer, var);
            let slice = buffer.finish(root, None);

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
            buffer.clear();
        }
    }
}
