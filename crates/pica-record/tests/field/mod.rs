use std::io::Cursor;

use pica_record::*;

#[test]
fn field_ref_new() {
    let subfield = SubfieldRef::try_from(('9', "040269019")).unwrap();
    let field = FieldRef::new("041A", None, vec![('9', "040269019")]);

    assert_eq!(field.tag(), b"041A");
    assert_eq!(field.occurrence(), None);
    assert_eq!(field.subfields(), &[subfield]);
}

#[test]
#[should_panic]
fn field_ref_new_panic() {
    let _field = FieldRef::new("041A", None, vec![('!', "040269019")]);
}

#[test]
fn field_ref_from_bytes() {
    let field =
        FieldRef::from_bytes(b"001A \x1f01140:20-11-22\x1e").unwrap();
    assert_eq!(field.tag(), b"001A");
}

#[test]
fn field_ref_try_from() {
    let bytes = "021A \x1faGrundlagen der Informationswissenschaft\x1e";
    let field = FieldRef::try_from(bytes.as_bytes()).unwrap();
    assert_eq!(field.subfields().len(), 1);
    assert_eq!(field.tag(), b"021A");

    let bytes = "02!A \x1fa123\x1e";
    let err = FieldRef::try_from(bytes.as_bytes()).unwrap_err();
    assert_eq!(err, ParsePicaError::InvalidField);
}

#[test]
fn field_ref_tag() {
    let field = FieldRef::new("041A", None, vec![('9', "040269019")]);
    assert_eq!(field.tag(), b"041A");
}

#[test]
fn field_ref_occurrence() {
    let field = FieldRef::new("041A", None, vec![]);
    assert_eq!(field.occurrence(), None);

    let occurrence = OccurrenceRef::new("01");
    let field = FieldRef::new("041A", Some("01"), vec![]);
    assert_eq!(field.occurrence(), Some(&occurrence));
}

#[test]
fn field_ref_subfields() {
    let subfield = SubfieldRef::try_from(('9', "040269019")).unwrap();
    let field = FieldRef::new("041A", None, vec![('9', "040269019")]);
    assert_eq!(field.subfields(), &[subfield]);

    let field = FieldRef::new("041A", None, vec![]);
    assert!(field.subfields().is_empty());
}

#[test]
fn field_ref_validate() {
    let field = FieldRef::from_bytes(b"019@ \x1fXA-DE-BE\x1e").unwrap();
    assert!(field.validate().is_ok());

    let field =
        FieldRef::from_bytes(b"019@ \x1f0\x00\x9F\x1e").unwrap();
    assert!(field.validate().is_err());
}

#[test]
fn field_ref_write_to() {
    let mut writer = Cursor::new(Vec::<u8>::new());
    let field =
        FieldRef::from_bytes(b"033A \x1fnDe Gruyter Saur\x1e").unwrap();
    let _ = field.write_to(&mut writer);
    assert_eq!(writer.into_inner(), b"033A \x1fnDe Gruyter Saur\x1e");

    let mut writer = Cursor::new(Vec::<u8>::new());
    let field =
        FieldRef::from_bytes(b"203@/01 \x1f0850439868\x1e").unwrap();
    let _ = field.write_to(&mut writer);
    assert_eq!(writer.into_inner(), b"203@/01 \x1f0850439868\x1e");
}

#[test]
fn field_ref_level() {
    let field = FieldRef::from_bytes(b"011@ \x1fa2022\x1e").unwrap();
    assert_eq!(field.level(), Level::Main);

    let field = FieldRef::from_bytes(b"101@ \x1fa1\x1e").unwrap();
    assert_eq!(field.level(), Level::Local);

    let field =
        FieldRef::from_bytes(b"203@/01 \x1f0850439868\x1e").unwrap();
    assert_eq!(field.level(), Level::Copy);
}

#[test]
fn field_ref_into_iter() {
    let field = FieldRef::from_bytes(b"002@ \x1f0Oaf\x1e").unwrap();
    let mut iter = field.into_iter();

    assert_eq!(iter.next(), Some(&field));
    assert_eq!(iter.next(), None);
}

#[test]
fn field_from_ref() {
    let field_ref =
        FieldRef::from_bytes(b"001U \x1f0utf8\x1e").unwrap();
    let _field = Field::from(field_ref);
}
