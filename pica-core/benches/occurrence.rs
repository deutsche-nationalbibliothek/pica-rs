use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

use pica_core::{Occurrence, OccurrenceRef};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Occurrence");
    group.warm_up_time(Duration::from_secs(10));
    group.measurement_time(Duration::from_secs(45));
    group.sample_size(10_000);
    group.bench_function("OccurrenceRef::from_bytes", |b| {
        b.iter(|| OccurrenceRef::from_bytes(black_box(b"/01")))
    });
    group.bench_function("Occurrence::from_bytes", |b| {
        b.iter(|| Occurrence::from_bytes(black_box(b"/01")))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
