use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

use pica_core::{Field, FieldRef};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Field");
    group.warm_up_time(Duration::from_secs(10));
    group.measurement_time(Duration::from_secs(30));
    group.bench_function("FieldRef::from_bytes", |b| {
        b.iter(|| {
            FieldRef::from_bytes(black_box(b"003@ \x1f0123456789X\x1fabc\x1e"))
        })
    });
    group.bench_function("Field::from_bytes", |b| {
        b.iter(|| {
            Field::from_bytes(black_box(b"003@ \x1f0123456789X\x1fabc\x1e"))
        })
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
