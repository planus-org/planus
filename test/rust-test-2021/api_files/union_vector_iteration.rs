use planus::ReadAsRoot;

#[track_caller]
#[allow(clippy::uninlined_format_args)]
fn eq<I1, I2>(iter1: I1, iter2: I2)
where
    I1: Clone + Iterator + ExactSizeIterator,
    I2: Clone + Iterator + ExactSizeIterator,
    I1::Item: core::fmt::Debug + PartialEq<I2::Item>,
    I2::Item: core::fmt::Debug,
{
    if !iter1.clone().eq(iter2.clone()) {
        let iter1 = iter1.collect::<Vec<_>>();
        let iter2 = iter2.collect::<Vec<_>>();
        panic!(
            "Not equal: {iter1:?} {iter2:?}",
            iter1 = iter1,
            iter2 = iter2
        );
    }
    let mut len = iter2.len();
    assert_eq!(len, iter1.len());
    let mut iter2 = iter2;
    while let Some(_) = iter2.next() {
        assert_eq!(iter2.len(), len - 1);
        len -= 1;
    }
}

fn to_owned_vec<'buf>(vec: planus::UnionVector<'buf, UnionRef<'buf>>) -> Vec<Union> {
    vec.iter().map(|v| v.unwrap().try_into().unwrap()).collect()
}

#[track_caller]
fn cmp<I1, I2, F, T>(iter1: I1, iter2: I2, f: F)
where
    I1: Clone + Iterator + DoubleEndedIterator + ExactSizeIterator,
    I2: Clone + Iterator + DoubleEndedIterator + ExactSizeIterator,
    I1::Item: core::fmt::Debug + PartialEq<T>,
    F: Copy + Fn(I2::Item) -> T,
    T: core::fmt::Debug,
{
    eq(iter1.clone(), iter2.clone().map(f));
    eq(iter1.clone().rev(), iter2.clone().rev().map(f));
    for i in 1..=iter1.len() + 1 {
        eq(iter1.clone().step_by(i), iter2.clone().step_by(i).map(f));
        eq(
            iter1.clone().step_by(i).rev(),
            iter2.clone().step_by(i).rev().map(f),
        );
        eq(
            iter1.clone().rev().step_by(i),
            iter2.clone().rev().step_by(i).map(f),
        );
    }
}

fn run_test(values: &[Union]) {
    let mut builder = planus::Builder::new();
    for i in 0..=values.len() {
        builder.clear();
        let root = Root::create(&mut builder, &values[..i]);
        let data = builder.finish(root, None);
        let root = RootRef::read_as_root(data).unwrap();
        let vector = root.unions().unwrap().unwrap();
        let owned: Vec<Union> = to_owned_vec(vector);
        assert_eq!(owned, &values[..i]);

        cmp(
            owned.iter().cloned(),
            vector.iter(),
            |u: Result<UnionRef<'_>, _>| -> Union { u.unwrap().try_into().unwrap() },
        );

        for i in 1..=values.len() + 1 {
            cmp(owned.chunks(i), vector.chunks(i), to_owned_vec);
            cmp(owned.chunks_exact(i), vector.chunks_exact(i), to_owned_vec);
            cmp(owned.rchunks(i), vector.rchunks(i), to_owned_vec);
            cmp(
                owned.rchunks_exact(i),
                vector.rchunks_exact(i),
                to_owned_vec,
            );
            cmp(owned.windows(i), vector.windows(i), to_owned_vec);
        }
    }
}

run_test(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12].map(|value| Union::Struct(Struct { value })));
run_test(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12].map(|value| Union::String(format!("{value}"))));
