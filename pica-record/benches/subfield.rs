use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use pica_record::SubfieldRef;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("SubfieldRef::from_bytes", |b| {
        b.iter(|| {
            SubfieldRef::from_bytes(black_box(b"\x1f0123456789X"))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
