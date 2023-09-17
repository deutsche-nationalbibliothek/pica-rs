use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use pica_record::{
    FieldMut, FieldRef, OccurrenceMut, OccurrenceRef, RecordMut,
    RecordRef, SubfieldMut, SubfieldRef, TagMut, TagRef,
};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("SubfieldRef::from_bytes", |b| {
        b.iter(|| {
            SubfieldRef::from_bytes(black_box(b"\x1f0123456789X"))
        })
    });

    c.bench_function("SubfieldMut::from_bytes", |b| {
        b.iter(|| {
            SubfieldMut::from_bytes(black_box(b"\x1f0123456789X"))
        })
    });

    c.bench_function("TagRef::from_bytes", |b| {
        b.iter(|| TagRef::from_bytes(black_box(b"003@")))
    });

    c.bench_function("TagMut::from_bytes", |b| {
        b.iter(|| TagMut::from_bytes(black_box(b"003@")))
    });

    c.bench_function("OccurrenceRef::from_bytes", |b| {
        b.iter(|| OccurrenceRef::from_bytes(black_box(b"/001")))
    });

    c.bench_function("OccurrenceMut::from_bytes", |b| {
        b.iter(|| OccurrenceMut::from_bytes(black_box(b"/001")))
    });

    c.bench_function("FieldRef::from_bytes", |b| {
        b.iter(|| {
            FieldRef::from_bytes(black_box(b"003@ \x1f0123456789X\x1e"))
        })
    });

    c.bench_function("FieldMut::from_bytes", |b| {
        b.iter(|| {
            FieldMut::from_bytes(black_box(b"003@ \x1f0123456789X\x1e"))
        })
    });

    c.bench_function("RecordRef::from_bytes", |b| {
        b.iter(|| {
            RecordRef::from_bytes(black_box(
                b"003@ \x1f0123456789X\x1e002@ \x1f0Olfo\x1e\n",
            ))
        })
    });

    c.bench_function("RecordMut::from_bytes", |b| {
        b.iter(|| {
            RecordMut::from_bytes(black_box(
                b"003@ \x1f0123456789X\x1e002@ \x1f0Olfo\x1e\n",
            ))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
