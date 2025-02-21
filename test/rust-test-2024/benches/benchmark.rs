fn main() {}

/*
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use planus::{Builder, Offset};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| 2 + 2));
}

fn bench_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("Serialize");
    for i in [10000].into_iter() {
        let mut builder = Builder::new();
        group.bench_with_input(BenchmarkId::new("planus", i), &i, |b, i| {
            b.iter(|| serialize_planus(&mut builder, *i))
        });
        let mut builder = flatbuffers::FlatBufferBuilder::new();
        group.bench_with_input(BenchmarkId::new("flatbuffers", i), &i, |b, i| {
            b.iter(|| serialize_flatbuffers(&mut builder, *i))
        });
    }
    group.finish();
}

fn serialize_flatbuffers(builder: &mut flatbuffers::FlatBufferBuilder, iterations: u64) {
    for _ in 0..iterations {
        builder.reset();
        let offset = flatc::MyTable3::create(builder, &flatc::MyTable3Args { x: 4 });
        let offset = builder.create_vector(&[offset]);
        let w_offset =
            flatc::MyTable3::create(builder, &flatc::MyTable3Args { x: 1337 }).as_union_value();
        let offset = flatc::MyTable::create(
            builder,
            &flatc::MyTableArgs {
                x: 3,
                y: true,
                z: Some(offset),
                numse: flatc::MyEnumse::Banaaaaaaaan,
                w_type: flatc::HelloUnion::y,
                w: Some(w_offset),
            },
        );
        let offset = flatc::MyTable2::create(
            builder,
            &flatc::MyTable2Args {
                x: 1,
                y: Some(&flatc::MyStruct::new(2, true, flatc::MyEnumse::Apple)),
                z: Some(offset),
            },
        );
        builder.finish(offset, None);
        builder.finished_data();
    }
}

fn serialize_planus(builder: &mut Builder, iterations: u64) {
    for _ in 0..iterations {
        builder.clear();
        let table3: &[Offset<MyTable3>] = &[MyTable3::create(builder, 4)];
        let w = MyTable3::create(builder, 1337);
        let w = HelloUnion::create_y(builder, w);
        let offset = MyTable::create(builder, 3, true, MyEnumse::Banaaaaaaaan, table3, Some(w));
        let offset = MyTable2::create(
            builder,
            1,
            MyStruct {
                foo: 2,
                bar: true,
                baz: MyEnumse::Apple,
            },
            offset,
        );
        builder.finish(offset, None);
    }
}

criterion_group!(benches, bench_serialize);
criterion_main!(benches);
 */
