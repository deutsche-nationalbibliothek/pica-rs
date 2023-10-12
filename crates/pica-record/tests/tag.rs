use pica_record::{Level, Tag};

type TestResult = anyhow::Result<()>;

#[test]
fn new() -> TestResult {
    let tag = Tag::new("002@");
    assert_eq!(tag, "002@");

    Ok(())
}

#[test]
#[should_panic]
fn new_panics_on_invalid_tag() {
    Tag::new("303@");
}

#[test]
fn from_bytes() -> TestResult {
    let tag = Tag::from_bytes(b"044H")?;
    assert_eq!(tag, Tag::new("044H"));

    Ok(())
}

#[test]
fn level() -> TestResult {
    assert_eq!(Tag::new("010@").level(), Level::Main);
    assert_eq!(Tag::new("103@").level(), Level::Local);
    assert_eq!(Tag::new("203@").level(), Level::Copy);

    Ok(())
}

#[test]
fn index() -> TestResult {
    let tag = Tag::from_bytes(b"012A")?;
    assert_eq!(tag[0], b'0');
    assert_eq!(tag[1], b'1');
    assert_eq!(tag[2], b'2');
    assert_eq!(tag[3], b'A');

    Ok(())
}

#[test]
fn to_string() -> TestResult {
    let tag = Tag::from_bytes(b"012A")?;
    assert_eq!(tag.to_string(), "012A");

    Ok(())
}
