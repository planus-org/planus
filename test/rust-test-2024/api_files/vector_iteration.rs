use planus::ReadAsRoot;

trait Testable<'buf>: 'static + Sized + std::fmt::Debug + PartialEq + Clone
{
    type Ref: Sized + std::fmt::Debug + planus::VectorRead<'buf>;
    fn serialize(builder: &mut planus::Builder, values: &[Self]) -> planus::Offset<Root>;
    fn deserialize(buf: RootRef<'buf>) -> planus::Vector<'buf, Self::Ref>;
    fn to_owned(v: Self::Ref) -> Self;
    fn to_owned_vec(vec: planus::Vector<'buf, Self::Ref>) -> Vec<Self> {
        vec.iter().map(|v| Self::to_owned(v)).collect()
    }
}

impl<'buf> Testable<'buf> for Struct {
    type Ref = StructRef<'buf>;

    fn serialize(builder: &mut planus::Builder, values: &[Struct]) -> planus::Offset<Root> {
        Root::create(builder, values, (), (), (), ())
    }

    fn deserialize(buf: RootRef<'buf>) -> planus::Vector<'buf, StructRef<'buf>> {
        buf.structs().unwrap().unwrap()
    }

    fn to_owned(v: StructRef<'buf>) -> Struct {
        v.into()
    }
}

impl<'buf> Testable<'buf> for Table {
    type Ref = planus::Result<TableRef<'buf>>;
    fn serialize(builder: &mut planus::Builder, values: &[Table]) -> planus::Offset<Root> {
        Root::create(builder, (), values, (), (), ())
    }

    fn deserialize(buf: RootRef<'buf>) -> planus::Vector<'buf, planus::Result<TableRef<'buf>>> {
        buf.tables().unwrap().unwrap()
    }

    fn to_owned(v: planus::Result<TableRef<'buf>>) -> Table {
        v.unwrap().try_into().unwrap()
    }
}

impl<'buf> Testable<'buf> for String {
    type Ref = planus::Result<&'buf str>;

    fn serialize(builder: &mut planus::Builder, values: &[String]) -> planus::Offset<Root> {
        Root::create(builder, (), (), values, (), ())
    }

    fn deserialize(buf: RootRef<'buf>) -> planus::Vector<'buf, planus::Result<&'buf str>> {
        buf.strings().unwrap().unwrap()
    }

    fn to_owned(v: planus::Result<&str>) -> String {
        v.unwrap().into()
    }
}

impl<'buf> Testable<'buf> for u16 {
    type Ref = u16;

    fn serialize(builder: &mut planus::Builder, values: &[u16]) -> planus::Offset<Root> {
        Root::create(builder, (), (), (), values, ())
    }

    fn deserialize(buf: RootRef<'buf>) -> planus::Vector<'buf, u16> {
        buf.uint16s().unwrap().unwrap()
    }

    fn to_owned(v: u16) -> u16 {
        v
    }
}

impl<'buf> Testable<'buf> for u64 {
    type Ref = u64;

    fn serialize(builder: &mut planus::Builder, values: &[u64]) -> planus::Offset<Root> {
        Root::create(builder, (), (), (), (), values)
    }

    fn deserialize(buf: RootRef<'buf>) -> planus::Vector<'buf, u64> {
        buf.uint64s().unwrap().unwrap()
    }

    fn to_owned(v: u64) -> u64 {
        v
    }
}

#[track_caller]
fn eq<I1, I2>(iter1: I1, iter2: I2)
where
    I1: Clone + Iterator + ExactSizeIterator,
    I2: Clone + Iterator + ExactSizeIterator,
    I1::Item: std::fmt::Debug + PartialEq<I2::Item>,
    I2::Item: std::fmt::Debug,
{
    if !iter1.clone().eq(iter2.clone()) {
        let iter1 = iter1.collect::<Vec<_>>();
        let iter2 = iter2.collect::<Vec<_>>();
        panic!("Not equal: {iter1:?} {iter2:?}");
    }
    let mut len = iter2.len();
    assert_eq!(len, iter1.len());
    let mut iter2 = iter2;
    while let Some(_) = iter2.next() {
        assert_eq!(iter2.len(), len - 1);
        len -= 1;
    }
}

#[track_caller]
fn cmp<I1, I2, F, T>(iter1: I1, iter2: I2, f: F)
where
    I1: Clone + Iterator + DoubleEndedIterator + ExactSizeIterator,
    I2: Clone + Iterator + DoubleEndedIterator + ExactSizeIterator,
    I1::Item: std::fmt::Debug + PartialEq<T>,
    F: Copy + Fn(I2::Item) -> T,
    T: std::fmt::Debug,
{
    eq(iter1.clone(), iter2.clone().map(f));
    eq(iter1.clone().rev(), iter2.clone().rev().map(f));
    for i in 1..=iter1.len()+1 {
        eq(iter1.clone().step_by(i), iter2.clone().step_by(i).map(f));
        eq(iter1.clone().step_by(i).rev(), iter2.clone().step_by(i).rev().map(f));
        eq(iter1.clone().rev().step_by(i), iter2.clone().rev().step_by(i).map(f));
    }
}

fn run_test<T>(values: &[T])
where
    for<'buf> T: Testable<'buf>,
{
    let mut builder = planus::Builder::new();
    for i in 0..=values.len() {
        builder.clear();
        let root = T::serialize(&mut builder, &values[..i]);
        let data = builder.finish(root, None);
        let root = RootRef::read_as_root(data).unwrap();
        let vector = T::deserialize(root);
        let owned = T::to_owned_vec(vector);
        assert_eq!(owned, &values[..i]);

        cmp(owned.iter().cloned(), vector.iter(), T::to_owned);

        for i in 1..=values.len()+1 {
            cmp(owned.chunks(i), vector.chunks(i), T::to_owned_vec);
            cmp(owned.chunks_exact(i), vector.chunks_exact(i), T::to_owned_vec);
            cmp(owned.rchunks(i), vector.rchunks(i), T::to_owned_vec);
            cmp(owned.rchunks_exact(i), vector.rchunks_exact(i), T::to_owned_vec);
            cmp(owned.windows(i), vector.windows(i), T::to_owned_vec);
        }
    }
}

run_test(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12].map(|value| Struct { value }));
run_test(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12].map(|value| Table { value }));
run_test(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12].map(|value| format!("{value}")));
run_test(&[1u16, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
run_test(&[1u64, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
