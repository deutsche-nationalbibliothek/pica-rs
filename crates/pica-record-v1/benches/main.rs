use std::path::Path;
use std::sync::OnceLock;
use std::{env, fs};

use criterion::*;
use pica_record_v1::{Record, RecordRef};
use quickcheck::{Arbitrary, Gen};

fn ada_lovelace() -> &'static [u8] {
    static DATA: OnceLock<Vec<u8>> = OnceLock::new();
    DATA.get_or_init(|| {
        let path = Path::new(&env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/data/ada.dat");
        fs::read_to_string(&path).unwrap().as_bytes().to_vec()
    })
}

fn arbitrary(size: usize) -> Vec<u8> {
    let record = Record::arbitrary(&mut Gen::new(size));
    let mut bytes = vec![];
    let _ = record.write_to(&mut bytes);
    bytes
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("RecordRef::from_bytes (ada)", |b| {
        b.iter(|| RecordRef::from_bytes(black_box(ada_lovelace())))
    });

    c.bench_function("RecordRef::from_bytes (arbitrary)", |b| {
        let bytes = arbitrary(100);
        b.iter(|| {
            let _record =
                RecordRef::from_bytes(black_box(&bytes)).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
