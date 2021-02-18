extern crate pica;

use pica::legacy::{Field, Occurrence, ParsePicaError, Record, Subfield};

#[test]
fn record_new() {
    let field =
        Field::new("003@", None, vec![Subfield::new('0', "123").unwrap()]);

    let record = Record::new(vec![field]);
    assert_eq!(record.len(), 1);
}

#[test]
fn record_len() {
    let field1 =
        Field::new("003@", None, vec![Subfield::new('0', "123").unwrap()]);
    let field2 =
        Field::new("002@", None, vec![Subfield::new('0', "Tp1").unwrap()]);

    let record = Record::new(vec![field1, field2]);
    assert_eq!(record.len(), 2);
}

#[test]
fn record_is_empty() {
    let field =
        Field::new("003@", None, vec![Subfield::new('0', "123").unwrap()]);

    let record = Record::new(vec![field]);
    assert!(!record.is_empty());

    let record = Record::new(vec![]);
    assert!(record.is_empty());
}

#[test]
fn record_decode() {
    let record: Record =
        Record::decode("003@ \u{1f}0123\u{1e}012A/00 \u{1f}a123\u{1e}")
            .unwrap();

    let field =
        Field::new("003@", None, vec![Subfield::new('0', "123").unwrap()]);
    assert!(record.contains(&field));

    let field = Field::new(
        "012A",
        Some(Occurrence::new("00")),
        vec![Subfield::new('a', "123").unwrap()],
    );
    assert!(record.contains(&field));
    assert_eq!(record.len(), 2);

    let result = Record::decode("003@ \u{1f}0123\u{1e}012A/00 \u{1f}a123");
    assert_eq!(result, Err(ParsePicaError::InvalidRecord));
}
