extern crate pica;

use pica::{Field, Filter, ParsePicaError, Path, Record, Subfield};

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
fn record_pretty() {
    let record = Record::new(vec![
        Field::new(
            "003@",
            None,
            vec![Subfield::new('0', "123456789").unwrap()],
        ),
        Field::new(
            "012A",
            Some("00"),
            vec![
                Subfield::new('a', "123").unwrap(),
                Subfield::new('b', "456").unwrap(),
            ],
        ),
    ]);

    assert_eq!(record.pretty(), "003@ $0 123456789\n012A/00 $a 123 $b 456");
}

#[test]
fn record_path() {
    let record = Record::decode(
        "012A \u{1f}a1\u{1f}a2\u{1f}b3\u{1e}012A \u{1f}a3\u{1e}",
    )
    .unwrap();

    let path = "012A.a".parse::<Path>().unwrap();
    assert_eq!(record.path(&path), vec!["1", "2", "3"]);

    let path = "012A.a[1]".parse::<Path>().unwrap();
    assert_eq!(record.path(&path), vec!["2"]);

    let path = "012A.a[9]".parse::<Path>().unwrap();
    assert!(record.path(&path).is_empty());
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
        Some("00"),
        vec![Subfield::new('a', "123").unwrap()],
    );
    assert!(record.contains(&field));
    assert_eq!(record.len(), 2);

    let result = Record::decode("003@ \u{1f}0123\u{1e}012A/00 \u{1f}a123");
    assert_eq!(result, Err(ParsePicaError::InvalidRecord));
}

#[test]
fn record_matches() {
    let record =
        Record::decode("003@ \u{1f}0123456789X\u{1e}012A \u{1f}a3\u{1e}")
            .unwrap();

    let filter = "003@.0 == '123456789X'".parse::<Filter>().unwrap();
    assert!(record.matches(&filter));

    let filter = "003@.0 != '123456789Y'".parse::<Filter>().unwrap();
    assert!(record.matches(&filter));

    let filter = "003@.0 == '123456789X' && 012A.a == '3'"
        .parse::<Filter>()
        .unwrap();
    assert!(record.matches(&filter));

    let filter = "003@.0 == '123456789X' && (012A.a == '4' || 012A.a == '3')"
        .parse::<Filter>()
        .unwrap();
    assert!(record.matches(&filter));
}
