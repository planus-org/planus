use std::{
    cmp::{Ordering, PartialOrd},
    f32::{INFINITY, NAN},
};

// The idea is to put all of the values into a tree
// * All values in leafs are comparable, unequal and written in order
// * For internal nodes, all left values are less than all right-values
// * For incomparable nodes, all of the values are incomparable to values
//   in that position in the tree and below (this includes themselves)
#[derive(Debug)]
enum Tree {
    Leaf(Vec<Union>),
    Node {
        left: Box<Tree>,
        right: Box<Tree>,
    },
    Incomparable {
        incomparable: Vec<Union>,
        inner: Box<Tree>,
    },
}

impl Tree {
    fn callback_iter<C>(&self, ctx: &mut C, f: impl Copy + Fn(&mut C, &Union)) {
        match self {
            Self::Leaf(cases) => {
                for case in cases {
                    f(ctx, case);
                }
            }
            Self::Node { left, right } => {
                left.callback_iter(ctx, f);
                right.callback_iter(ctx, f);
            }
            Self::Incomparable {
                incomparable,
                inner,
            } => {
                for case in incomparable {
                    f(ctx, case);
                }
                inner.callback_iter(ctx, f);
            }
        }
    }

    fn run(&self, comparisons: &mut u32) {
        match self {
            Self::Leaf(cases) => {
                for (i, v0) in cases.iter().enumerate() {
                    for (j, v1) in cases.iter().enumerate() {
                        *comparisons += 1;
                        if i == j {
                            assert_eq!(v0, v1);
                        } else {
                            assert_ne!(v0, v1);
                        }
                        assert_eq!(v0.partial_cmp(v1), i.partial_cmp(&j));
                    }
                }
            }
            Self::Node { left, right } => {
                left.callback_iter(comparisons, |comparisons, v0| {
                    right.callback_iter(comparisons, |comparisons, v1| {
                        *comparisons += 2;
                        assert_ne!(v0, v1);
                        assert_ne!(v1, v0);
                        assert_eq!(
                            v0.partial_cmp(v1),
                            Some(Ordering::Less),
                            "Expected {v0:?} < {v1:?}"
                        );
                        assert_eq!(
                            v1.partial_cmp(v0),
                            Some(Ordering::Greater),
                            "Expected {v1:?} > {v0:?}"
                        );
                    });
                });
                left.run(comparisons);
                right.run(comparisons);
            }
            Self::Incomparable {
                incomparable,
                inner,
            } => {
                for v0 in incomparable {
                    for v1 in incomparable {
                        *comparisons += 1;
                        assert_ne!(v0, v1);
                        assert_eq!(v0.partial_cmp(v1), None);
                    }
                }

                inner.callback_iter(comparisons, |comparisons, v0| {
                    for v1 in incomparable {
                        *comparisons += 2;
                        assert_ne!(v0, v1);
                        assert_ne!(v1, v0);
                        assert_eq!(v0.partial_cmp(v1), None);
                        assert_eq!(v1.partial_cmp(v0), None);
                    }
                });
                inner.run(comparisons);
            }
        }
    }
}

let f0_cases = Tree::Leaf(vec![
    Union::F0("".into()),
    Union::F0("hello0".into()),
    Union::F0("hello1".into()),
]);
let f1_cases = Tree::Leaf(vec![
    Union::F1(InnerU8 { v0: 0, v1: 0 }),
    Union::F1(InnerU8 { v0: 0, v1: 1 }),
    Union::F1(InnerU8 { v0: 1, v1: 0 }),
    Union::F1(InnerU8 { v0: 1, v1: 1 }),
]);
let f2_cases = Tree::Leaf(vec![
    Union::F2(Box::new(InnerTable { v0: 0, v1: 0 })),
    Union::F2(Box::new(InnerTable { v0: 0, v1: 1 })),
    Union::F2(Box::new(InnerTable { v0: 1, v1: 0 })),
    Union::F2(Box::new(InnerTable { v0: 1, v1: 1 })),
]);
let f3_cases0 = Tree::Incomparable {
    incomparable: vec![Union::F3(InnerF32 { v0: 0.0, v1: NAN })],
    inner: Box::new(Tree::Leaf(vec![
        Union::F3(InnerF32 { v0: 0.0, v1: -10.0 }),
        Union::F3(InnerF32 { v0: 0.0, v1: 0.0 }),
        Union::F3(InnerF32 { v0: 0.0, v1: 10.0 }),
        Union::F3(InnerF32 {
            v0: 0.0,
            v1: INFINITY,
        }),
    ])),
};
let f3_cases1 = Tree::Incomparable {
    incomparable: vec![Union::F3(InnerF32 { v0: 1.0, v1: NAN })],
    inner: Box::new(Tree::Leaf(vec![
        Union::F3(InnerF32 { v0: 1.0, v1: -10.0 }),
        Union::F3(InnerF32 { v0: 1.0, v1: 0.0 }),
        Union::F3(InnerF32 { v0: 1.0, v1: 10.0 }),
        Union::F3(InnerF32 {
            v0: 1.0,
            v1: INFINITY,
        }),
    ])),
};
let f3_cases = Tree::Incomparable {
    incomparable: vec![
        Union::F3(InnerF32 { v0: NAN, v1: 0.0 }),
        Union::F3(InnerF32 { v0: NAN, v1: 1.0 }),
    ],
    inner: Box::new(Tree::Node {
        left: Box::new(f3_cases0),
        right: Box::new(f3_cases1),
    }),
};
let inner_u8_cases = Tree::Leaf(vec![
    Union::InnerU8(InnerU8 { v0: 0, v1: 0 }),
    Union::InnerU8(InnerU8 { v0: 0, v1: 1 }),
    Union::InnerU8(InnerU8 { v0: 1, v1: 0 }),
    Union::InnerU8(InnerU8 { v0: 1, v1: 1 }),
]);

let all_cases = [f0_cases, f1_cases, f2_cases, f3_cases, inner_u8_cases]
    .into_iter()
    .reduce(|left, right| Tree::Node {
        left: Box::new(left),
        right: Box::new(right),
    })
    .unwrap();

let mut test_cases = 0;
all_cases.callback_iter(&mut test_cases, |test_cases, _v| {
    *test_cases += 1;
});
let mut comparisons = 0;
all_cases.run(&mut comparisons);
assert_eq!(comparisons, test_cases * test_cases);
