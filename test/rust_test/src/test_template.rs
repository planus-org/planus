// Included by build.rs into test files, should not be added to lib.rs
#[allow(unused_imports)]
use planus::{SliceWithStartOffset, TableRead, WriteAsOffset};
#[allow(unused_imports)]
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

#[test]
fn test_serialize() {
    let json = std::fs::read_to_string(format!("{}.json", FILE_PATH)).unwrap();
    let root: Root = serde_json::from_str(&json).unwrap();

    let bin_path = format!("{}.bin", FILE_PATH);
    let data = if let Ok(mut file) = OpenOptions::new().read(true).write(true).open(&bin_path) {
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        data
    } else {
        let mut buffer = planus::Buffer::new();
        let root = root.prepare(&mut buffer);
        let data = buffer.finish(root, None);
        File::create(&bin_path).unwrap().write_all(data).unwrap();
        data.to_vec()
    };

    let root_ref = RootRef::from_buffer(
        SliceWithStartOffset {
            buffer: &data,
            offset_from_start: 0,
        },
        0,
    )
    .unwrap();
    let root2 = planus::ToOwned::to_owned(root_ref).unwrap();
    assert_eq!(root, root2);
}
