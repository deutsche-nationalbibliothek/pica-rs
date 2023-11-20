use std::fmt::Write;
use std::io::Cursor;

use pica_record::{
    ByteRecord, ParsePicaError, Record, RecordRef, StringRecord,
};

#[test]
fn record_ref_new() {
    let record = RecordRef::new(vec![
        ("003@", None, vec![('0', "123237025")]),
        ("028A", None, vec![('d', "Rainer"), ('a', "Kuhlen")]),
    ]);

    assert_eq!(record.iter().len(), 2);
}

#[test]
#[should_panic]
fn record_ref_new_panic() {
    let _record =
        RecordRef::new(vec![("00!@", None, vec![('0', "123237025")])]);
}

#[test]
fn record_ref_from_bytes() {
    let record =
        RecordRef::from_bytes(b"041R \x1faProf. Dr.\x1f4akad\x1e\n")
            .unwrap();
    assert_eq!(record.iter().len(), 1);

    let err =
        RecordRef::from_bytes(b"041R \x1faProf. Dr.\x1f!akad\x1e\n")
            .unwrap_err();
    assert!(matches!(err, ParsePicaError::InvalidRecord(_)));
}

#[test]
fn record_ref_is_empty() {
    let record =
        RecordRef::from_bytes(b"041R \x1faProf. Dr.\x1f4akad\x1e\n")
            .unwrap();
    assert!(!record.is_empty());

    let fields: Vec<(&str, Option<&str>, Vec<(char, &str)>)> = vec![];
    let record = RecordRef::new(fields);
    assert!(record.is_empty());
}

#[test]
fn record_ref_iter() {
    let record =
        RecordRef::from_bytes(b"041R \x1faProf. Dr.\x1f4akad\x1e\n")
            .unwrap();
    let mut iter = record.iter();

    assert!(iter.next().is_some());
    assert!(iter.next().is_none());
}

#[test]
fn record_ref_retain() {
    let bytes = b"003@ \x1f0123456789X\x1e012A \x1fa123\x1e\n";
    let mut record = RecordRef::from_bytes(bytes).unwrap();
    assert_eq!(record.iter().len(), 2);

    record.retain(|field| field.tag() == b"012A");
    assert_eq!(record.iter().len(), 1);

    record.retain(|field| field.tag() == b"003@");
    assert!(record.is_empty());
}

#[test]
fn record_ref_validate() {
    let bytes = b"003@ \x1f0123456789X\x1e012A \x1fa123\x1e\n";
    let record = RecordRef::from_bytes(bytes).unwrap();
    assert!(record.validate().is_ok());

    let bytes = b"003@ \x1f0\x00\x9F\x1e012A \x1fa123\x1e\n";
    let record = RecordRef::from_bytes(bytes).unwrap();
    assert!(record.validate().is_err());
}

#[test]
fn record_ref_write_to() {
    let mut writer = Cursor::new(Vec::<u8>::new());
    let record = RecordRef::from_bytes(b"003@ \x1f0123\x1e\n").unwrap();
    let _ = record.write_to(&mut writer);

    assert_eq!(writer.into_inner(), b"003@ \x1f0123\x1e\n");
}

#[test]
fn record_from_ref() {
    let record_ref =
        RecordRef::from_bytes(b"003@ \x1f0123\x1e\n").unwrap();
    let _record = Record::from(record_ref);
}

#[test]
fn byte_record_from_bytes() {
    let record =
        ByteRecord::from_bytes(b"041R \x1faProf. Dr.\x1f4akad\x1e\n")
            .unwrap();
    assert_eq!(record.iter().len(), 1);

    let err =
        ByteRecord::from_bytes(b"041R \x1faProf. Dr.\x1f!akad\x1e\n")
            .unwrap_err();
    assert!(matches!(err, ParsePicaError::InvalidRecord(_)));
}

#[test]
fn byte_record_write_to() {
    let mut writer = Cursor::new(Vec::<u8>::new());
    let record =
        ByteRecord::from_bytes(b"003@ \x1f0123\x1e\n").unwrap();
    let _ = record.write_to(&mut writer);

    assert_eq!(writer.into_inner(), b"003@ \x1f0123\x1e\n");
}

#[test]
fn byte_record_retain() {
    let bytes = b"003@ \x1f0123456789X\x1e012A \x1fa123\x1e\n";
    let mut record = ByteRecord::from_bytes(bytes).unwrap();
    assert_eq!(record.iter().len(), 2);

    record.retain(|field| field.tag() == b"012A");
    assert_eq!(record.iter().len(), 1);

    record.retain(|field| field.tag() == b"003@");
    assert!(record.is_empty());
}

#[test]
fn byte_record_hash() {
    let bytes = b"003@ \x1f0123456789X\x1e012A \x1fa123\x1e\n";
    let record = ByteRecord::from_bytes(bytes).unwrap();

    let hash =
        record.sha256().iter().fold(String::new(), |mut out, b| {
            let _ = write!(out, "{b:02x}");
            out
        });

    let expected = "f9bf144682fe03f32b2ad2d4048c84a1\
                    2a4d58cb557dd8f44066ae7d81cebd5c";
    assert_eq!(hash, expected);
}

#[test]
fn byte_record_from_ref() {
    let bytes = b"003@ \x1f0123456789X\x1e012A \x1fa123\x1e\n";
    let record_ref = RecordRef::from_bytes(bytes).unwrap();
    let record = ByteRecord::from(record_ref);
    assert_eq!(record.iter().len(), 2);
}

#[test]
fn string_record_try_from() {
    let bytes = b"003@ \x1f0123456789X\x1e012A \x1fa123\x1e\n";
    let byte_record = ByteRecord::from_bytes(bytes).unwrap();
    let record = StringRecord::try_from(byte_record).unwrap();
    assert_eq!(record.iter().len(), 2);
}

#[test]
fn string_record_from_bytes() {
    let record =
        StringRecord::from_bytes(b"041R \x1faProf. Dr.\x1f4akad\x1e\n")
            .unwrap();
    assert_eq!(record.iter().len(), 1);

    let err =
        StringRecord::from_bytes(b"041R \x1faProf. Dr.\x1f!akad\x1e\n")
            .unwrap_err();
    assert!(matches!(err, ParsePicaError::InvalidRecord(_)));
}

#[test]
fn string_record_retain() {
    let bytes = b"003@ \x1f0123456789X\x1e012A \x1fa123\x1e\n";
    let mut record = StringRecord::from_bytes(bytes).unwrap();
    assert_eq!(record.iter().len(), 2);

    record.retain(|field| field.tag() == b"012A");
    assert_eq!(record.iter().len(), 1);

    record.retain(|field| field.tag() == b"003@");
    assert!(record.is_empty());
}

#[test]
fn string_record_hash() {
    let bytes = b"003@ \x1f0123456789X\x1e012A \x1fa123\x1e\n";
    let record = StringRecord::from_bytes(bytes).unwrap();

    let hash =
        record.sha256().iter().fold(String::new(), |mut out, b| {
            let _ = write!(out, "{b:02x}");
            out
        });

    let expected = "f9bf144682fe03f32b2ad2d4048c84a1\
                    2a4d58cb557dd8f44066ae7d81cebd5c";
    assert_eq!(hash, expected);
}
