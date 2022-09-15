use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use pica_record::{OccurrenceRef, SubfieldRef, TagRef};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("SubfieldRef::from_bytes", |b| {
        b.iter(|| {
            SubfieldRef::from_bytes(black_box(b"\x1f0123456789X"))
        })
    });

    c.bench_function("TagRef::from_bytes", |b| {
        b.iter(|| TagRef::from_bytes(black_box(b"003@")))
    });

    c.bench_function("OccurrenceRef::from_bytes", |b| {
        b.iter(|| OccurrenceRef::from_bytes(black_box(b"/001")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
