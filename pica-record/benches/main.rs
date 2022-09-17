use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use pica_record::{Subfield, SubfieldRef};
// use pica_record::{FieldRef, OccurrenceRef, SubfieldRef, TagRef};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("SubfieldRef::from_bytes", |b| {
        b.iter(|| {
            SubfieldRef::from_bytes(black_box(b"\x1f0123456789X"))
        })
    });

    c.bench_function("Subfield::from_bytes", |b| {
        b.iter(|| Subfield::from_bytes(black_box(b"\x1f0123456789X")))
    });

    // c.bench_function("TagRef::from_bytes", |b| {
    //     b.iter(|| TagRef::from_bytes(black_box(b"003@")))
    // });

    // c.bench_function("OccurrenceRef::from_bytes", |b| {
    //     b.iter(|| OccurrenceRef::from_bytes(black_box(b"/001")))
    // });

    // c.bench_function("FieldRef::from_bytes", |b| {
    //     b.iter(|| {
    //         FieldRef::from_bytes(black_box(b"003@
    // \x1f0123456789X\x1e"))     })
    // });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
