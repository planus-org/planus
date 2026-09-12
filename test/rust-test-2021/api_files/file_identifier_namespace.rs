use planus::ReadAsRoot;

// The identifier constant is exposed on the namespaced table type.
assert_eq!(my::pkg::Msg::IDENTIFIER, *b"NSP1");

let mut builder = planus::Builder::new();
let msg = my::pkg::Msg::builder()
    .x(7)
    .label("hello")
    .finish(&mut builder);
let bytes = builder
    .finish(msg, Some(my::pkg::Msg::IDENTIFIER))
    .to_vec();

assert_eq!(&bytes[4..8], b"NSP1");
assert!(planus::buffer_has_identifier(
    &bytes,
    my::pkg::Msg::IDENTIFIER
));

let read = my::pkg::MsgRef::read_as_root(&bytes).unwrap();
assert_eq!(read.x().unwrap(), 7);
assert_eq!(read.label().unwrap(), Some("hello"));
