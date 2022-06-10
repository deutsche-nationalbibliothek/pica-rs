use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

use pica_core::{Subfield, SubfieldRef};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Subfield");
    group.warm_up_time(Duration::from_secs(10));
    group.measurement_time(Duration::from_secs(45));
    group.sample_size(10_000);
    group.bench_function("SubfieldRef::from_bytes", |b| {
        b.iter(|| SubfieldRef::from_bytes(black_box(b"\x1f0123456789X")))
    });
    group.bench_function("Subfield::from_bytes", |b| {
        b.iter(|| Subfield::from_bytes(black_box(b"\x1f0123456789X")))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
