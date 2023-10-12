use std::io::Cursor;

use bstr::ByteSlice;
use pica_record::{Occurrence, ParsePicaError};

type TestResult = anyhow::Result<()>;

#[test]
fn new() -> TestResult {
    let occurrence = Occurrence::new("00");
    assert_eq!(occurrence, "00");

    Ok(())
}

#[test]
#[should_panic]
fn new_panic_on_invalid_occurrence() {
    Occurrence::new("0A");
}

#[test]
fn from_bytes() -> TestResult {
    let occurrence = Occurrence::from_bytes(b"/001")?;
    assert_eq!(occurrence, Occurrence::new("001"));

    assert_eq!(
        Occurrence::from_bytes(b"/00A").unwrap_err(),
        ParsePicaError::InvalidOccurrence
    );

    Ok(())
}

#[test]
fn write_to() -> TestResult {
    let mut writer = Cursor::new(Vec::<u8>::new());
    let occurrence = Occurrence::new("002");
    occurrence.write_to(&mut writer)?;

    assert_eq!(String::from_utf8(writer.into_inner())?, "/002");

    Ok(())
}

#[test]
fn to_string() -> TestResult {
    let occurrence = Occurrence::new("03");
    assert_eq!(occurrence.to_string(), String::from("03"));

    Ok(())
}

#[test]
fn try_from() -> TestResult {
    assert_eq!(
        Occurrence::try_from("04".as_bytes().as_bstr()).unwrap(),
        Occurrence::new("04")
    );

    assert_eq!(
        Occurrence::try_from(b"0!".as_bstr()).unwrap_err(),
        ParsePicaError::InvalidOccurrence
    );

    Ok(())
}
