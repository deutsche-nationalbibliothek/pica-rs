use pica::ByteRecord;

#[test]
fn record_from_bytes() {
    let _record =
        ByteRecord::from_bytes(b"012A/00 \x1fa123456789\x1e".to_vec()).unwrap();

    assert!(ByteRecord::from_bytes(b"".to_vec()).is_err());
    assert!(ByteRecord::from_bytes(b"012! \x1f123".to_vec()).is_err());
    assert!(ByteRecord::from_bytes(b"012A!\x1f123".to_vec()).is_err());
    assert!(ByteRecord::from_bytes(b"012A !123".to_vec()).is_err());
    assert!(ByteRecord::from_bytes(b"012A\x1e".to_vec()).is_err());
}
