// This is a bit overly complicated because of incomparable values.
// All values in the inner lists are in comparable to each other and in order
// The option values are incomparable to the associated list, but comparable to
// all other values
let values: Vec<(Vec<Union>, Option<Union>)> = vec![
    (vec![
        Union::F0("".into()),
        Union::F0("hello1".into()),
        Union::F0("hello2".into()),
        Union::F1(InnerU8 { value: 0 }),
        Union::F1(InnerU8 { value: 1 }),
    ], None),
    (vec![
        Union::F2(InnerF32 { value: -10.0 }),
        Union::F2(InnerF32 { value: 10.0 }),
        Union::F2(InnerF32 { value: std::f32::INFINITY }),
    ], Some(Union::F2(InnerF32 { value: std::f32::NAN }))),
    (vec![
        Union::F3(Box::new(InnerTable { value: 0 })),
        Union::F3(Box::new(InnerTable { value: 1 })),
        Union::InnerU8(InnerU8 { value: 0 }),
        Union::InnerU8(InnerU8 { value: 1 }),
    ], None),
    (vec![
        Union::InnerF32(InnerF32 { value: -10.0 }),
        Union::InnerF32(InnerF32 { value: 10.0 }),
        Union::InnerF32(InnerF32 { value: std::f32::INFINITY }),
    ], Some(Union::InnerF32(InnerF32 { value: std::f32::NAN }))),
    (vec![
        Union::InnerTable(Box::new(InnerTable { value: 0 })),
        Union::InnerTable(Box::new(InnerTable { value: 1 })),
    ], None),
];

use std::cmp::{PartialOrd, Ordering};

let normal = values.iter().enumerate().flat_map(|(i, (l, _v))| l.iter().enumerate().map(move |(j, v)| ((i, j), v)));
let special = values.iter().enumerate().flat_map(|(i, (_l, v))| v.iter().map(move |v| (i, v)));

// Do all the (normal, normal) pairs
for (i, v0) in normal.clone() {
    for (j, v1) in normal.clone() {
        if i == j {
            assert_eq!(v0, v1);
        } else {
            assert_ne!(v0, v1);
        }
        assert_eq!(v0.partial_cmp(v1), i.partial_cmp(&j));
    }
}

// Do all the (normal, special) and (special, normal) pairs
for ((i, _), v0) in normal.clone() {
    for (j, v1) in special.clone() {
        assert_ne!(v0, v1);
        assert_ne!(v1, v0);
        if i == j {
            assert_eq!(v0.partial_cmp(v1), None);
            assert_eq!(v1.partial_cmp(v0), None);
        } else {
            assert_eq!(v0.partial_cmp(v1), i.partial_cmp(&j));
            assert_eq!(v1.partial_cmp(v0), j.partial_cmp(&i));
        }
    }
}

// Do all the (special, special) pairs
for (i, v0) in special.clone() {
    for (j, v1) in special.clone() {
        assert_ne!(v0, v1);
        if i == j {
            assert_eq!(v0.partial_cmp(v1), None);
        } else {
            assert_eq!(v0.partial_cmp(v1), i.partial_cmp(&j));
        }
    }
}
