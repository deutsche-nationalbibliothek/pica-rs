use std::io::Cursor;

use pica_record::{ParsePicaError, Subfield, SubfieldRef};

#[test]
fn subfield_ref_new() {
    let subfield = SubfieldRef::new('a', "abc");
    assert_eq!(subfield.code(), 'a');
    assert_eq!(subfield.value(), "abc");
}

#[test]
#[should_panic]
fn subfield_ref_new_panic() {
    let _ = SubfieldRef::new('!', "abc");
}

#[test]
fn subfield_ref_from_bytes() {
    let subfield = SubfieldRef::from_bytes(b"\x1f0abc").unwrap();
    assert_eq!(subfield.code(), '0');
    assert_eq!(subfield.value(), "abc");
}

#[test]
fn subfield_ref_try_from() {
    let subfield = SubfieldRef::try_from(('a', "abc")).unwrap();
    assert_eq!(subfield.code(), 'a');
    assert_eq!(subfield.value(), "abc");

    let err = SubfieldRef::try_from(('!', "abc")).unwrap_err();
    assert_eq!(err, ParsePicaError::InvalidSubfield);

    let err = SubfieldRef::try_from(('a', "a\x1fc")).unwrap_err();
    assert_eq!(err, ParsePicaError::InvalidSubfield);

    let err = SubfieldRef::try_from(('a', "a\x1ec")).unwrap_err();
    assert_eq!(err, ParsePicaError::InvalidSubfield);
}

#[test]
fn subfield_ref_code() {
    let subfield = SubfieldRef::new('1', "abc");
    assert_eq!(subfield.code(), '1');
}

#[test]
fn subfield_ref_value() {
    let subfield = SubfieldRef::new('1', "abc");
    assert_eq!(subfield.value(), "abc");
}

#[test]
fn subfield_ref_is_empty() {
    let subfield = SubfieldRef::new('1', "abc");
    assert!(!subfield.is_empty());

    let subfield = SubfieldRef::new('1', "");
    assert!(subfield.is_empty());
}

#[test]
fn subfield_ref_validate() {
    let subfield = SubfieldRef::new('1', "abc");
    assert!(subfield.validate().is_ok());

    let subfield =
        SubfieldRef::from_bytes(&[b'\x1f', b'0', 0, 159]).unwrap();
    assert!(subfield.validate().is_err());
}

#[test]
fn subfield_ref_write_to() {
    let mut writer = Cursor::new(Vec::<u8>::new());
    let subfield = SubfieldRef::new('0', "abcdef");
    let _ = subfield.write_to(&mut writer);
    assert_eq!(writer.into_inner(), b"\x1f0abcdef");
}

#[test]
fn subfield_ref_into_iter() {
    let subfield = SubfieldRef::new('0', "abcdef");
    let mut iter = subfield.into_iter();

    assert_eq!(iter.next(), Some(&subfield));
    assert_eq!(iter.next(), None);
}

#[test]
fn subfield_ref_partial_eq() {
    let subfield_ref = SubfieldRef::new('0', "abc");
    let subfield = Subfield::from(subfield_ref.clone());
    assert_eq!(subfield_ref, subfield);
}

#[test]
fn subfield_write_to() {
    let mut writer = Cursor::new(Vec::<u8>::new());
    let subfield: Subfield = SubfieldRef::new('0', "abcdef").into();
    let _ = subfield.write_to(&mut writer);
    assert_eq!(writer.into_inner(), b"\x1f0abcdef");
}

#[test]
fn subfield_from_ref() {
    let subfield_ref = SubfieldRef::new('0', "abc");
    let _subfield = Subfield::from(subfield_ref.clone());
}

#[test]
fn subfield_partial_eq() {
    let subfield_ref = SubfieldRef::new('0', "abc");
    let subfield = Subfield::from(subfield_ref.clone());
    assert_eq!(subfield, subfield_ref);
}
