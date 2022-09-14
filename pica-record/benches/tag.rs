use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use pica_record::TagRef;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("TagRef::from_bytes", |b| {
        b.iter(|| TagRef::from_bytes(black_box(b"003@")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
