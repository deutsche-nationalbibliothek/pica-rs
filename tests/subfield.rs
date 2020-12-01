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
