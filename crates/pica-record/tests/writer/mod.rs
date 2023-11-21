use pica_record::io::{
    Reader, ReaderBuilder, RecordsIterator, WriterBuilder,
};
use pica_record::ByteRecord;
use tempfile::NamedTempFile;

#[test]
fn writer_builder_from_path() {
    let bytes = b"003@ \x1f0123456789X\x1e012A \x1fa123\x1e\n";
    let record = ByteRecord::from_bytes(bytes).unwrap();

    let tempfile = NamedTempFile::new().unwrap();
    let mut writer =
        WriterBuilder::new().from_path(tempfile.path()).unwrap();
    assert!(writer.write_byte_record(&record).is_ok());
    assert!(writer.finish().is_ok());

    let mut reader: Reader<_> =
        ReaderBuilder::new().from_path(tempfile.path()).unwrap();

    let mut count = 0;
    while let Some(result) = reader.next() {
        assert_eq!(result.unwrap(), record);
        count += 1;
    }

    assert_eq!(count, 1);
}

#[test]
fn writer_builder_from_path_or_stdout() {
    let bytes = b"003@ \x1f0123456789X\x1e012A \x1fa123\x1e\n";
    let record = ByteRecord::from_bytes(bytes).unwrap();

    let tempfile = NamedTempFile::new().unwrap();
    let mut writer = WriterBuilder::new()
        .from_path_or_stdout(Some(tempfile.path()))
        .unwrap();
    assert!(writer.write_byte_record(&record).is_ok());
    assert!(writer.finish().is_ok());

    let mut reader: Reader<_> =
        ReaderBuilder::new().from_path(tempfile.path()).unwrap();

    let mut count = 0;
    while let Some(result) = reader.next() {
        assert_eq!(result.unwrap(), record);
        count += 1;
    }

    assert_eq!(count, 1);
}

#[test]
fn writer_builder_gzip() {
    let bytes = b"003@ \x1f0123456789X\x1e012A \x1fa123\x1e\n";
    let record = ByteRecord::from_bytes(bytes).unwrap();

    let tempfile = NamedTempFile::new().unwrap();
    let mut path = tempfile.path().to_str().unwrap().to_string();
    path.push_str(".gz");

    let mut writer =
        WriterBuilder::new().gzip(true).from_path(&path).unwrap();
    assert!(writer.write_byte_record(&record).is_ok());
    assert!(writer.finish().is_ok());

    let mut reader: Reader<_> =
        ReaderBuilder::new().from_path(&path).unwrap();

    let mut count = 0;
    while let Some(result) = reader.next() {
        assert_eq!(result.unwrap(), record);
        count += 1;
    }

    assert_eq!(count, 1);
}

#[test]
fn writer_builder_append() {
    let bytes = b"003@ \x1f0123456789X\x1e012A \x1fa123\x1e\n";
    let record = ByteRecord::from_bytes(bytes).unwrap();

    let tempfile = NamedTempFile::new().unwrap();

    let mut writer =
        WriterBuilder::new().from_path(tempfile.path()).unwrap();
    assert!(writer.write_byte_record(&record).is_ok());
    assert!(writer.finish().is_ok());

    let mut writer = WriterBuilder::new()
        .append(true)
        .from_path(tempfile.path())
        .unwrap();
    assert!(writer.write_byte_record(&record).is_ok());
    assert!(writer.finish().is_ok());

    let mut reader: Reader<_> =
        ReaderBuilder::new().from_path(tempfile.path()).unwrap();

    let mut count = 0;
    while let Some(result) = reader.next() {
        assert_eq!(result.unwrap(), record);
        count += 1;
    }

    assert_eq!(count, 2);
}
