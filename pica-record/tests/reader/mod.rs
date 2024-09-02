use std::io::Cursor;
use std::path::PathBuf;

use pica_record::io::{ReaderBuilder, RecordsIterator};

#[test]
fn reader_builder_limit() {
    let data = Cursor::new(
        b"003@ \x1f0123456789X\x1e012A \x1fa123\x1e\n\
        003@ \x1f0123456789X\x1e012A \x1fa123\x1e\n",
    );

    let mut reader =
        ReaderBuilder::new().limit(1).from_reader(data, None);

    let mut count = 0;
    while let Some(result) = reader.next() {
        let _record = result.unwrap();
        count += 1;
    }

    assert_eq!(count, 1);
}

#[test]
fn reader_builder_from_reader() {
    let data =
        Cursor::new(b"003@ \x1f0123456789X\x1e012A \x1fa123\x1e\n");
    let mut reader = ReaderBuilder::new().from_reader(data, None);
    assert!(reader.next().is_some());
    assert!(reader.next().is_none());
}

#[test]
fn reader_builder_from_path() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/data/DUMP.dat.gz");

    let mut reader = ReaderBuilder::new().from_path(path).unwrap();
    let mut count = 0;
    while let Some(result) = reader.next() {
        if result.is_ok() {
            count += 1;
        }
    }

    assert_eq!(count, 12);
}
