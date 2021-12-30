#[cfg(test)]
mod tests {
    use planus::{Builder, ReadAsRoot};

    use crate::planus::enums::*;

    #[test]
    fn test_roundtrip() {
        let mut builder = Builder::new();

        for var in [Abc::A, Abc::B, Abc::C] {
            let root = Wrap::create(&mut builder, var);
            let slice = builder.finish(root, None);

            let table = WrapRef::read_as_root(slice).unwrap();
            println!("{:?}", table);

            assert_eq!(table.abc().unwrap(), var);
            builder.clear();
        }
    }
}
