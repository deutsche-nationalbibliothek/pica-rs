use winnow::prelude::*;
use winnow::token::one_of;

use crate::SubfieldCode;

/// Parse a PICA+ subfield code.
pub fn parse_subfield_code(i: &mut &[u8]) -> PResult<SubfieldCode> {
    one_of((b'0'..=b'9', b'a'..=b'z', b'A'..=b'Z'))
        .map(SubfieldCode::from_unchecked)
        .parse_next(i)
}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;

    use super::*;

    #[quickcheck]
    fn test_parse_arbitrary_subfield_code(code: u8) {
        if code.is_ascii_alphanumeric() {
            assert_eq!(
                parse_subfield_code.parse(&[code]).unwrap(),
                SubfieldCode::from_unchecked(char::from(code))
            );
        } else {
            assert!(parse_subfield_code.parse(&[code]).is_err());
        }
    }
}
