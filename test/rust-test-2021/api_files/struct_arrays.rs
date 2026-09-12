use planus::ReadAsRoot;

let id = Uuid {
    uuid: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
};
let cov = Mat {
    data: [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
};
let mix = Mixed {
    a: 200,
    vals: [1.5, 2.5, 3.5, 4.5],
    b: 9_000_000_000,
};

let mut builder = planus::Builder::new();
let holder = Holder::builder()
    .id(id)
    .cov(cov)
    .mix(mix)
    .finish(&mut builder);
let bytes = builder.finish(holder, None).to_vec();

let read = HolderRef::read_as_root(&bytes).unwrap();
assert_eq!(read.id().unwrap().uuid(), id.uuid);
assert_eq!(read.cov().unwrap().data(), cov.data);
let m = read.mix().unwrap();
assert_eq!(m.a(), 200);
assert_eq!(m.vals(), [1.5, 2.5, 3.5, 4.5]);
assert_eq!(m.b(), 9_000_000_000);

// Owned conversion preserves the arrays.
let owned: Holder = read.try_into().unwrap();
assert_eq!(owned.id.uuid, id.uuid);
assert_eq!(owned.cov.data, cov.data);
assert_eq!(owned.mix.vals[3], 4.5);
