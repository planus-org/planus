// Included by build.rs into test files, should not be added to lib.rs
#[allow(unused_imports)]
use planus::{ReadAsRoot, WriteAsOffset};
#[allow(unused_imports)]
use std::convert::identity;
#[allow(unused_imports)]
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

#[test]
fn test_serialize() {
    let should_regenerate = std::env::var("PLANUS_REGENERATE").is_ok();
    for entry in std::fs::read_dir(format!("{}/{}", FILE_PATH, "serialize")).unwrap() {
        let entry = entry.unwrap();
        let file_path = entry.path();
        if !file_path.is_dir()
            && file_path
                .extension()
                .map_or(false, |extension| extension == "json")
        {
            let json = std::fs::read_to_string(&file_path).unwrap();
            let root: Root = serde_json::from_str(&json).unwrap();

            let mut builder = planus::Builder::new();
            let offset = root.prepare(&mut builder);
            let data = builder.finish(offset, None);

            let root_ref = RootRef::read_as_root(data).unwrap();
            let root2 = Root::try_from(root_ref).unwrap();
            similar_asserts::assert_eq!(root, root2);

            let mut bin_path = file_path.clone();
            bin_path.set_extension("bin");
            crate::tests::compare_regenerate_file(&bin_path, data, should_regenerate).unwrap();

            let mut dump_path = file_path.clone();
            dump_path.set_extension("dump.txt");
            let dump = crate::hexdump::hexdump_flatbuffer_table(data);
            crate::tests::compare_regenerate_file_str(&dump_path, &dump, should_regenerate)
                .unwrap();

            let mut dbg_path = file_path.clone();
            dbg_path.set_extension("dbg.txt");
            let flatc_dbg = flatbuffers::root::<flatc::Root>(data).unwrap();
            let flatc_dbg = format!("{:#?}", flatc_dbg);
            crate::tests::compare_regenerate_file_str(&dbg_path, &flatc_dbg, should_regenerate)
                .unwrap();
        }
    }
}

#[test]
fn test_deserialize() {
    if let Ok(refs_dir) = std::fs::read_dir(format!("{}/{}", FILE_PATH, "deserialize")) {
        let should_regenerate = std::env::var("PLANUS_REGENERATE").is_ok();
        for entry in refs_dir {
            let entry = entry.unwrap();
            let file_path = entry.path();
            if file_path
                .extension()
                .map_or(false, |extension| extension == "bin")
            {
                let data = std::fs::read(&file_path).unwrap();

                let root_ref = RootRef::read_as_root(&data).unwrap();

                let mut debug_path = file_path.clone();
                debug_path.set_extension("txt");
                let root_dbg = format!("{:#?}", root_ref);

                crate::tests::compare_regenerate_file_str(
                    &debug_path,
                    &root_dbg,
                    should_regenerate,
                )
                .unwrap();
            }
        }
    }
}
