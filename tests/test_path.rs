extern crate pica;

use pica::{Occurrence, ParsePicaError, Path};

#[test]
fn path_new() {
    let path = Path::new("012A", Some(Occurrence::new("00")), '0', Some(0));
    assert_eq!(path.tag(), "012A");
    assert_eq!(path.occurrence(), Some(&Occurrence::new("00")));
    assert_eq!(path.name(), '0');
    assert_eq!(path.index(), Some(0));
}

#[test]
fn path_to_string() {
    let path = Path::new("012A", Some(Occurrence::new("00")), '0', Some(1));
    assert_eq!(path.to_string(), "012A/00.0[1]");

    let path = Path::new("012A", None, '0', Some(1));
    assert_eq!(path.to_string(), "012A.0[1]");

    let path = Path::new("012A", None, '0', None);
    assert_eq!(path.to_string(), "012A.0");
}

#[test]
fn path_from_str() {
    let path = Path::decode(" 012A/00.0[1] ").unwrap();
    assert_eq!(path.tag(), "012A");
    assert_eq!(path.occurrence(), Some(&Occurrence::new("00")));
    assert_eq!(path.name(), '0');
    assert_eq!(path.index(), Some(1));

    let path = Path::decode("012A/00.0").unwrap();
    assert_eq!(path.tag(), "012A");
    assert_eq!(path.occurrence(), Some(&Occurrence::new("00")));
    assert_eq!(path.name(), '0');
    assert_eq!(path.index(), None);

    let path = Path::decode("012A.0").unwrap();
    assert_eq!(path.tag(), "012A");
    assert_eq!(path.occurrence(), None);
    assert_eq!(path.name(), '0');
    assert_eq!(path.index(), None);

    let path = Path::decode("012A");
    assert_eq!(path, Err(ParsePicaError::InvalidPath));
}
