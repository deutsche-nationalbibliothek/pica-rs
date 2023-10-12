use pica_record::{Field, ParsePicaError, Subfield};

type TestResult = anyhow::Result<()>;

#[test]
fn new() -> TestResult {
    let field = Field::new("012A", None, vec![('a', "123")]);
    assert_eq!(field.tag(), b"012A");
    assert_eq!(field.occurrence(), None);

    let mut iter = field.subfields().iter();
    assert_eq!(iter.next(), Some(&Subfield::new('a', "123")));
    assert_eq!(iter.next(), None);

    Ok(())
}

#[test]
#[should_panic]
fn new_panics() {
    Field::new("312A", None, vec![('a', "123")]);
}

#[test]
fn from_bytes() -> TestResult {
    assert_eq!(
        Field::from_bytes(b"012A/01 \x1fa123\x1e")?,
        Field::new("012A", Some("01"), vec![('a', "123")],),
    );

    assert_eq!(
        Field::from_bytes(b"012A/01 \x1fa123").unwrap_err(),
        ParsePicaError::InvalidField
    );

    Ok(())
}
