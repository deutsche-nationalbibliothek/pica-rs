extern crate pica;

use pica::{ParsePicaError, Subfield};

#[test]
fn subfield_new() {
    for code in '\0'..='\u{7e}' {
        let result = Subfield::new(code, "abc");

        if !code.is_ascii_alphanumeric() {
            assert_eq!(result, Err(ParsePicaError::InvalidSubfield));
        } else {
            let subfield = result.unwrap();
            assert_eq!(subfield.code(), code);
            assert_eq!(subfield.value(), "abc");
        }
    }
}

// #[test]
// fn subfield_pretty() {
//     let subfield = Subfield::new('0', "abc").unwrap();
//     assert_eq!(subfield.pretty(), "$0 abc");
// }

#[test]
fn subfield_parse() {
    let subfield = "\u{1f}0abc".parse::<Subfield>().unwrap();
    assert_eq!(subfield.code(), '0');
    assert_eq!(subfield.value(), "abc");

    let result = "\u{1f}!abc".parse::<Subfield>();
    assert_eq!(result.err(), Some(ParsePicaError::InvalidSubfield));
}
