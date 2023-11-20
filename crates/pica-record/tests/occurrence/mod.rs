use std::io::Cursor;

use pica_record::{Occurrence, OccurrenceRef, ParsePicaError};

#[test]
fn occurrence_ref_new() {
    assert_eq!(OccurrenceRef::new("00"), "00");
    assert_eq!(OccurrenceRef::new("000"), "000");
    assert_eq!(OccurrenceRef::new("001"), "001");
    assert_eq!(OccurrenceRef::new("01"), "01");
    assert_eq!(OccurrenceRef::new("99"), "99");
    assert_eq!(OccurrenceRef::new("999"), "999");
}

#[test]
#[should_panic]
fn occurrence_ref_new_panic() {
    let _ = OccurrenceRef::new("/0A");
}

#[test]
fn occurrence_ref_from_bytes() {
    assert_eq!(OccurrenceRef::from_bytes(b"/00").unwrap(), "00");
    assert_eq!(OccurrenceRef::from_bytes(b"/000").unwrap(), "000");
    assert_eq!(OccurrenceRef::from_bytes(b"/001").unwrap(), "001");
    assert_eq!(OccurrenceRef::from_bytes(b"/01").unwrap(), "01");
    assert_eq!(OccurrenceRef::from_bytes(b"/99").unwrap(), "99");
    assert_eq!(OccurrenceRef::from_bytes(b"/999").unwrap(), "999");

    let err = OccurrenceRef::from_bytes(b"/0A").unwrap_err();
    assert_eq!(err, ParsePicaError::InvalidOccurrence);
}

#[test]
fn occurrence_ref_try_from() {
    for o in ["00", "000", "001", "01", "99", "999"] {
        assert_eq!(OccurrenceRef::try_from(o.as_bytes()).unwrap(), o);
    }

    let err = OccurrenceRef::try_from("0A".as_bytes()).unwrap_err();
    assert_eq!(err, ParsePicaError::InvalidOccurrence);
}

#[test]
fn occurrence_ref_as_bytes() {
    let occurrence = OccurrenceRef::new("01");
    assert_eq!(occurrence.as_bytes(), b"01");
}

#[test]
fn occurrence_ref_from_unchecked() {
    let occurrence = OccurrenceRef::from_unchecked("01".into());
    assert_eq!(occurrence, "01");
}

#[test]
fn occurrence_ref_write_to() {
    let mut writer = Cursor::new(Vec::<u8>::new());
    let ocurrence = OccurrenceRef::new("001");
    let _ = ocurrence.write_to(&mut writer);
    assert_eq!(writer.into_inner(), b"/001");
}

#[test]
fn occurrence_ref_partial_eq() {
    let occurrence = OccurrenceRef::new("001");
    assert_eq!(occurrence, b"001");
    assert_eq!(occurrence, "001");
}

#[test]
fn occurrence_ref_to_string() {
    let occurrence = OccurrenceRef::new("001");
    assert_eq!(occurrence.to_string(), "/001".to_string());
}

#[test]
fn occurrence_as_ref() {
    let occurrence_ref = OccurrenceRef::new("001");
    let occurrence = Occurrence::from(occurrence_ref);
    assert_eq!(occurrence.as_ref(), b"001");
}

#[test]
fn occurrence_as_bytes() {
    let occurrence_ref = OccurrenceRef::new("001");
    let occurrence = Occurrence::from(occurrence_ref);
    assert_eq!(occurrence.as_bytes(), b"001");
}
