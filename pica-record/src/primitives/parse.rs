use winnow::combinator::preceded;
use winnow::prelude::*;
use winnow::token::{one_of, take_till};

use super::SubfieldRef;
use crate::{SubfieldCode, SubfieldValueRef};

/// Parse a PICA+ subfield code.
pub fn parse_subfield_code(i: &mut &[u8]) -> PResult<SubfieldCode> {
    one_of((b'0'..=b'9', b'a'..=b'z', b'A'..=b'Z'))
        .map(SubfieldCode::from_unchecked)
        .parse_next(i)
}

/// Parse a PICA+ subfield value reference.
pub fn parse_subfield_value_ref<'a>(
    i: &mut &'a [u8],
) -> PResult<SubfieldValueRef<'a>> {
    take_till(0.., |c| c == b'\x1f' || c == b'\x1e')
        .map(SubfieldValueRef::from_unchecked)
        .parse_next(i)
}

/// Parse a PICA+ subfield.
pub fn parse_subfield_ref<'a>(
    i: &mut &'a [u8],
) -> PResult<SubfieldRef<'a>> {
    preceded(b'\x1f', (parse_subfield_code, parse_subfield_value_ref))
        .map(|(code, value)| SubfieldRef { code, value })
        .parse_next(i)
}

#[cfg(test)]
mod tests {
    use bstr::ByteSlice;
    use quickcheck_macros::quickcheck;

    use super::*;
    use crate::Subfield;

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

    #[test]
    fn test_parse_subfield_value_ref() {
        macro_rules! parse_success {
            ($input:expr, $expected:expr, $rest:expr) => {
                let value = SubfieldValueRef::from_unchecked($expected);
                assert_eq!(
                    parse_subfield_value_ref
                        .parse_peek($input)
                        .unwrap(),
                    ($rest.as_bytes(), value)
                );
            };
        }
        parse_success!(b"abc", b"abc", b"");
        parse_success!(b"a\x1ebc", b"a", b"\x1ebc");
        parse_success!(b"a\x1fbc", b"a", b"\x1fbc");
        parse_success!(b"", b"", b"");
    }

    #[quickcheck]
    fn test_parse_arbitrary_subfield_value_ref(input: String) {
        let input = input.replace(['\x1f', '\x1e'], "");
        let rest = b"".as_bytes();

        let value = SubfieldValueRef::from_unchecked(input.as_bytes());
        assert_eq!(
            parse_subfield_value_ref
                .parse_peek(input.as_bytes())
                .unwrap(),
            (rest, value)
        );
    }

    #[test]
    fn test_parse_subfield_ref() {
        assert_eq!(
            parse_subfield_ref.parse(b"\x1fa123").unwrap(),
            SubfieldRef::new('a', "123").unwrap()
        );

        assert_eq!(
            parse_subfield_ref.parse(b"\x1fa").unwrap(),
            SubfieldRef::new('a', "").unwrap()
        );

        assert!(parse_subfield_ref.parse(b"a123").is_err());
        assert!(parse_subfield_ref.parse(b"").is_err());
    }

    #[cfg_attr(miri, ignore)]
    #[quickcheck_macros::quickcheck]
    fn test_parse_arbitrary_subfield_ref(subfield: Subfield) -> bool {
        let mut bytes = Vec::<u8>::new();
        let _ = subfield.write_to(&mut bytes);
        parse_subfield_ref.parse(&bytes).is_ok()
    }
}
