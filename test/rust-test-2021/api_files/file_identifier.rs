use planus::ReadAsRoot;

// The schema's file_identifier is exposed as an associated constant.
assert_eq!(Message::IDENTIFIER, *b"MSG1");

let mut builder = planus::Builder::new();
let message = Message::builder().x(2.5).n(42).tag(7).finish(&mut builder);
let bytes = builder.finish(message, Some(Message::IDENTIFIER)).to_vec();

// The identifier is written into bytes 4..8, matching the flatbuffers wire format.
assert_eq!(&bytes[4..8], b"MSG1");
assert!(planus::buffer_has_identifier(&bytes, Message::IDENTIFIER));
assert!(!planus::buffer_has_identifier(&bytes, *b"XXXX"));

let read = MessageRef::read_as_root(&bytes).unwrap();
assert_eq!(read.x().unwrap(), 2.5);
assert_eq!(read.n().unwrap(), 42);
assert_eq!(read.tag().unwrap(), 7);
