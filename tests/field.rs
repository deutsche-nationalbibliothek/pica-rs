extern crate pica;

use pica::ParsePicaError;
use pica::{Field, Subfield};

#[test]
fn field_new() {
    let subfields = vec![Subfield::new('0', "123456789X").unwrap()];
    let field = Field::new("003@", None, subfields.clone());
    assert_eq!(field.tag(), "003@");
    assert_eq!(field.occurrence(), None);
    assert_eq!(field.subfields().len(), 1);

    let subfield = field.subfields().first().unwrap();
    assert_eq!(subfield.name(), '0');
    assert_eq!(subfield.value(), "123456789X");
}

#[test]
fn field_pretty() {
    let subfields = vec![Subfield::new('x', "abc").unwrap()];
    let field = Field::new("012A", Some("00"), subfields.clone());
    assert_eq!(field.pretty(), "012A/00 $x abc");
}

#[test]
fn field_parse() {
    let field = Field::decode("012A/00 \u{1f}0abc\u{1f}0def\u{1e}").unwrap();
    assert_eq!(field.tag(), "012A");
    assert_eq!(field.occurrence(), Some("00"));
    assert_eq!(field.subfields().len(), 2);

    let subfield = field.subfields().iter().nth(0).unwrap();
    assert_eq!(subfield.name(), '0');
    assert_eq!(subfield.value(), "abc");

    let subfield = field.subfields().iter().nth(1).unwrap();
    assert_eq!(subfield.name(), '0');
    assert_eq!(subfield.value(), "def");

    let result = Field::decode("012A/00 \u{1f}0abc\u{1f}0def");
    assert_eq!(result.err(), Some(ParsePicaError::InvalidField));
}
