#[cfg(test)]
mod tests {
    use crate::flatc::conformance as flatc;
    use crate::planus::conformance::*;
    use planus::{Buffer, BufferWithStartOffset, Offset, TableRead};

    #[test]
    fn test_simple() {
        let mut builder = flatbuffers::FlatBufferBuilder::new();
        let offset = flatc::MyTable3::create(&mut builder, &flatc::MyTable3Args { x: 4 });
        let offset = builder.create_vector(&[offset]);
        let w_offset = flatc::MyTable3::create(&mut builder, &flatc::MyTable3Args { x: 1337 })
            .as_union_value();
        let offset = flatc::MyTable::create(
            &mut builder,
            &flatc::MyTableArgs {
                x: 3,
                y: true,
                z: Some(offset),
                numse: flatc::MyEnumse::Banaaaaaaaan,
                w_type: flatc::HelloUnion::y,
                w: Some(w_offset),
            },
        );
        let offset = flatc::MyTable2::create(
            &mut builder,
            &flatc::MyTable2Args {
                x: 1,
                y: Some(&flatc::MyStruct::new(2, true, flatc::MyEnumse::Apple)),
                z: Some(offset),
            },
        );
        builder.finish(offset, None);
        let flatc_data = builder.finished_data();

        let mut buffer = Buffer::new();
        let foo: &[Offset<MyTable3>] = &[MyTable3::create(&mut buffer, 4)];
        let w = MyTable3::create(&mut buffer, 1337);
        let w = HelloUnion::create_y(&mut buffer, w);
        let offset = MyTable::create(
            &mut buffer,
            3,
            true,
            MyEnumse::Banaaaaaaaan,
            &foo[..],
            Some(w),
        );
        let offset = MyTable2::create(
            &mut buffer,
            1,
            MyStruct {
                foo: 2,
                bar: true,
                baz: MyEnumse::Apple,
            },
            offset,
        );
        let slice = buffer.finish(offset, None);
        let table = unsafe { flatbuffers::root_unchecked::<flatc::MyTable2>(slice) };
        let table = MyTable2Ref::from_buffer(
            BufferWithStartOffset {
                buffer: slice,
                offset_from_start: 0,
            },
            0,
        )
        .unwrap();
        let table = MyTable2Ref::from_buffer(
            BufferWithStartOffset {
                buffer: flatc_data,
                offset_from_start: 0,
            },
            0,
        )
        .unwrap();

        // let mut buffer = Buffer::new();
        // let w = MyTable3::create(&mut buffer, 1337);
        // let w = HelloUnion::create_y(&mut buffer, w);
        // let slice = buffer.finish(w, None);
        // println!("{:?}", slice);

        let mut builder = flatbuffers::FlatBufferBuilder::new();
        let x_offset = flatc::MyTable3::create(&mut builder, &flatc::MyTable3Args { x: 1337 })
            .as_union_value();
        let offset = flatc::MyTable4::create(
            &mut builder,
            &flatc::MyTable4Args {
                x_type: flatc::HelloUnion::y,
                x: Some(x_offset),
            },
        );
        builder.finish(offset, None);
    }
}
