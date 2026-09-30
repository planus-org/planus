use planus::ReadAsRoot;

// Member values are bit positions: value == 1 << position (matching flatc).
assert_eq!(Fault::A.0, 1);
assert_eq!(Fault::B.0, 2);
assert_eq!(Fault::C.0, 1024);

assert_eq!(Fault::empty().0, 0);
assert_eq!(Fault::all().0, 1 | 2 | 1024);
assert!(Fault::empty().is_empty());

let combo = Fault::A | Fault::C;
assert_eq!(combo.0, 1025);
assert!(combo.contains(Fault::A));
assert!(!combo.contains(Fault::B));
assert!(combo.intersects(Fault::C));
assert!(!combo.intersects(Fault::B));

// `!` masks back to the known bits.
assert_eq!((!Fault::A).0, 1026);

// Infallible conversions to/from the underlying integer.
assert_eq!(Fault::from(2u16), Fault::B);
assert_eq!(u16::from(Fault::C), 1024u16);

// Round-trip through a buffer as a scalar field and a vector element.
let mut builder = planus::Builder::new();
let holder = Holder::builder()
    .flags(combo)
    .list(vec![Fault::B, Fault::C])
    .finish(&mut builder);
let bytes = builder.finish(holder, None).to_vec();

let read = HolderRef::read_as_root(&bytes).unwrap();
assert_eq!(read.flags().unwrap(), combo);
let list: Vec<Fault> = read.list().unwrap().unwrap().to_vec().unwrap();
assert_eq!(list, vec![Fault::B, Fault::C]);
