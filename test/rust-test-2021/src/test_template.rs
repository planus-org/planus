// Included by build.rs into test files, should not be added to lib.rs
#[allow(unused_imports)]
use planus::{ReadAsRoot, WriteAsOffset};
#[allow(unused_imports)]
#[cfg(feature = "std")]
use std::{
    convert::identity,
    io::{Read, Write},
};

#[cfg(feature = "std")]
#[test]
fn test_serialize() {
    let should_regenerate = std::env::var("PLANUS_REGENERATE").is_ok();
    for entry in std::fs::read_dir(format!("{}/{}", FILE_PATH, "serialize")).unwrap() {
        let entry = entry.unwrap();
        let file_path = entry.path();
        if !file_path.is_dir()
            && file_path
                .extension()
                .is_some_and(|extension| extension == "json")
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
            let mut dump_path = file_path.clone();

            if cfg!(feature = "vtable-cache") && FILE_PATH.contains("vtable") {
                dump_path.set_extension("vtable-cache.dump.txt");
                bin_path.set_extension("vtable-cache.bin");
            } else if cfg!(feature = "string-cache") && FILE_PATH.contains("string") {
                dump_path.set_extension("string-cache.dump.txt");
                bin_path.set_extension("string-cache.bin");
            } else if cfg!(feature = "bytes-cache") && FILE_PATH.contains("byte") {
                dump_path.set_extension("bytes-cache.dump.txt");
                bin_path.set_extension("bytes-cache.bin");
            } else {
                dump_path.set_extension("dump.txt");
                bin_path.set_extension("bin");
            }

            crate::tests::compare_regenerate_file(&bin_path, data, should_regenerate).unwrap();

            let dump = crate::hexdump::hexdump_flatbuffer_table(data);
            crate::tests::compare_regenerate_file_str(&dump_path, &dump, should_regenerate)
                .unwrap();

            let mut dbg_rust_path = file_path.clone();
            dbg_rust_path.set_extension("rust-dbg.txt");
            let rust_dbg = format!("{root_ref:#?}");
            crate::tests::compare_regenerate_file_str(&dbg_rust_path, &rust_dbg, should_regenerate)
                .unwrap();

            // <FLATC>
            let mut dbg_path = file_path.clone();
            dbg_path.set_extension("dbg.txt");
            let flatc_dbg = flatbuffers::root::<flatc::Root>(data).unwrap();
            let flatc_dbg = format!("{flatc_dbg:#?}");
            crate::tests::compare_regenerate_file_str(&dbg_path, &flatc_dbg, should_regenerate)
                .unwrap();
            // </FLATC>
        }
    }
}

#[cfg(feature = "std")]
#[test]
fn test_deserialize() {
    if let Ok(refs_dir) = std::fs::read_dir(format!("{}/{}", FILE_PATH, "deserialize")) {
        let should_regenerate = std::env::var("PLANUS_REGENERATE").is_ok();
        for entry in refs_dir {
            let entry = entry.unwrap();
            let file_path = entry.path();
            if file_path
                .extension()
                .is_some_and(|extension| extension == "bin")
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
