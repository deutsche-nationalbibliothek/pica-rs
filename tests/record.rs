use pica::{ByteRecord, Field, Occurrence, Subfield};

#[test]
fn subfield() {
    let subfield = Subfield::new('0', "123456789X").unwrap();
    assert_eq!(subfield.code(), '0');
    assert_eq!(subfield.value(), "123456789X");

    for code in '\x00'..='\x7F' {
        let result = Subfield::new(code as char, "123456789X");
        match code {
            'a'..='z' | 'A'..='Z' | '0'..='9' => assert!(result.is_ok()),
            _ => assert!(result.is_err()),
        }
    }

    // record or unit separator are not allowed in subfield values.
    assert!(Subfield::new('0', "123\x1e45679").is_err());
    assert!(Subfield::new('0', "123\x1f45679").is_err());
}

#[test]
fn occurrence() {
    assert_eq!(Occurrence::new("00").unwrap(), "00");
    assert_eq!(Occurrence::new("001").unwrap(), "001");

    // invalid occurrences
    assert!(Occurrence::new("0").is_err());
    assert!(Occurrence::new("0000").is_err());
    assert!(Occurrence::new("A0").is_err());
    assert!(Occurrence::new("0A").is_err());
    assert!(Occurrence::new("00A").is_err());
}

#[test]
fn field() {
    let field = Field::new(
        "012A",
        Some(Occurrence::new("00").unwrap()),
        vec![Subfield::new('0', "123456789X").unwrap()],
    )
    .unwrap();

    assert_eq!(field.len(), 1);
    assert_eq!(field.occurrence(), Some(&Occurrence::new("00").unwrap()));
    assert_eq!(field.contains_code('0'), true);
    assert_eq!(
        field.get('0'),
        Some(vec![&Subfield::new('0', "123456789X").unwrap()])
    );
    assert_eq!(field.get('1'), None);
}

#[test]
fn record() {
    let haskell_curry_str = include_str!("../tests/data/12283643X.dat");
    let _record = ByteRecord::from_bytes(haskell_curry_str.as_bytes()).unwrap();

    assert!(ByteRecord::from_bytes(b"".to_vec()).is_err());
    assert!(ByteRecord::from_bytes(b"012! \x1f123".to_vec()).is_err());
    assert!(ByteRecord::from_bytes(b"012A!\x1f123".to_vec()).is_err());
    assert!(ByteRecord::from_bytes(b"012A !123".to_vec()).is_err());
    assert!(ByteRecord::from_bytes(b"012A\x1e".to_vec()).is_err());
}
