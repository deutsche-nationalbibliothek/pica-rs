use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

use pica_core::{Tag, TagRef};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tag");
    group.warm_up_time(Duration::from_secs(10));
    group.measurement_time(Duration::from_secs(45));
    group.sample_size(10_000);
    group.bench_function("TagRef::from_bytes", |b| {
        b.iter(|| TagRef::from_bytes(black_box(b"003@")))
    });
    group.bench_function("Tag::from_bytes", |b| {
        b.iter(|| Tag::from_bytes(black_box(b"003@")))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
