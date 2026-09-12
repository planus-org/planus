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

// Finishing without an identifier writes only the 4-byte root offset, so the
// identifier check fails while the payload still reads back correctly.
let mut builder = planus::Builder::new();
let message = Message::builder().x(2.5).n(42).tag(7).finish(&mut builder);
let bytes_no_id = builder.finish(message, None).to_vec();
assert!(!planus::buffer_has_identifier(&bytes_no_id, Message::IDENTIFIER));
let read = MessageRef::read_as_root(&bytes_no_id).unwrap();
assert_eq!(read.x().unwrap(), 2.5);

// <FLATC>
// Compatibility with the upstream `flatbuffers` crate (via flatc-generated code).

// The identifier constant agrees with the one flatc derives from the schema.
assert_eq!(&Message::IDENTIFIER, flatc::MESSAGE_IDENTIFIER.as_bytes());

// flatbuffers accepts and reads a buffer produced by planus.
assert!(flatc::message_buffer_has_identifier(&bytes));
let flatc_read = flatc::root_as_message(&bytes).unwrap();
assert_eq!(flatc_read.x(), 2.5);
assert_eq!(flatc_read.n(), 42);
assert_eq!(flatc_read.tag(), 7);

// A buffer built by flatbuffers (with the identifier) is accepted and read by planus,
// and its identifier region is byte-for-byte identical to the planus-produced buffer.
let mut fbb = flatbuffers::FlatBufferBuilder::new();
let flatc_message = flatc::Message::create(
    &mut fbb,
    &flatc::MessageArgs {
        x: 2.5,
        n: 42,
        tag: 7,
    },
);
flatc::finish_message_buffer(&mut fbb, flatc_message);
let flatc_bytes = fbb.finished_data();

assert_eq!(&bytes[4..8], &flatc_bytes[4..8]);
assert!(planus::buffer_has_identifier(flatc_bytes, Message::IDENTIFIER));
let read = MessageRef::read_as_root(flatc_bytes).unwrap();
assert_eq!(read.x().unwrap(), 2.5);
assert_eq!(read.n().unwrap(), 42);
assert_eq!(read.tag().unwrap(), 7);
// </FLATC>
