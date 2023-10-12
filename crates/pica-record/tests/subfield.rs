use std::io::Cursor;

use pica_record::{ParsePicaError, Subfield};

type TestResult = anyhow::Result<()>;

#[test]
fn subfield_new() -> anyhow::Result<()> {
    let subfield = Subfield::new('0', b"118540238");
    assert_eq!(subfield.value(), "118540238");
    assert_eq!(subfield.code(), '0');
    Ok(())
}

#[test]
#[should_panic]
fn subfield_new_panics_invalid_code() {
    Subfield::new('!', b"foobar");
}

#[test]
#[should_panic]
fn subfield_new_panics_value_rs() {
    Subfield::new('a', b"f\x1eoo");
}

#[test]
#[should_panic]
fn subfield_new_panics_value_us() {
    Subfield::new('a', b"f\x1foo");
}

#[test]
fn subfield_from_bytes() -> anyhow::Result<()> {
    let subfield = Subfield::from_bytes(b"\x1f0118540238")?;
    assert_eq!(subfield.value(), "118540238");
    assert_eq!(subfield.code(), '0');

    let error = Subfield::from_bytes(b"0118540238").unwrap_err();
    assert_eq!(error, ParsePicaError::InvalidSubfield);
    Ok(())
}

#[test]
fn subfield_is_empty() -> anyhow::Result<()> {
    let subfield = Subfield::new('0', b"118540238");
    assert!(!subfield.is_empty());

    let subfield = Subfield::new('0', b"");
    assert!(subfield.is_empty());

    Ok(())
}

#[test]
fn subfield_validate() -> anyhow::Result<()> {
    let subfield = Subfield::new('0', b"118540238");
    assert!(subfield.validate().is_ok());

    let subfield = Subfield::new('0', &[b'0', b'0', 159]);
    assert!(subfield.validate().is_err());

    Ok(())
}

#[test]
fn subfield_write_to() -> anyhow::Result<()> {
    let mut writer = Cursor::new(Vec::<u8>::new());
    let subfield = Subfield::new('0', b"118540238");
    subfield.write_to(&mut writer)?;

    assert_eq!(writer.into_inner(), b"\x1f0118540238");

    Ok(())
}

#[test]
fn subfield_into_iter() -> TestResult {
    let subfield = Subfield::new('0', b"118540238");
    let mut iter = subfield.into_iter();

    assert_eq!(iter.next(), Some(&subfield));
    assert_eq!(iter.next(), None);

    Ok(())
}

#[test]
fn subfield_try_from() -> TestResult {
    let subfield = Subfield::try_from(('a', "abc"))?;
    assert_eq!(subfield, Subfield::from_bytes(b"\x1faabc")?);

    assert!(Subfield::try_from(('a', "a\x1fb")).is_err());
    assert!(Subfield::try_from(('a', "a\x1eb")).is_err());
    assert!(Subfield::try_from(('!', "abc")).is_err());

    Ok(())
}
