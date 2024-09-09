use winnow::combinator::{opt, preceded, repeat};
use winnow::prelude::*;
use winnow::token::{one_of, take_till};

use super::{FieldRef, SubfieldRef};
use crate::occurrence::parse_occurrence;
use crate::tag::parse_tag;
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

/// Parse a PICA+ field.
pub fn parse_field_ref<'a>(i: &mut &'a [u8]) -> PResult<FieldRef<'a>> {
    (
        parse_tag,
        opt(parse_occurrence),
        b' ',
        repeat(0.., parse_subfield_ref),
        b'\x1e',
    )
        .map(|(tag, occurrence, _, subfields, _)| FieldRef {
            tag,
            occurrence,
            subfields,
        })
        .parse_next(i)
}

#[cfg(test)]
mod tests {
    use bstr::ByteSlice;
    use quickcheck_macros::quickcheck;

    use super::*;
    use crate::{Field, Subfield};

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

    #[test]
    fn test_parse_field_ref() {
        macro_rules! parse_success {
            ($i:expr, $tag:expr, $occurrence:expr, $subfields:expr) => {
                let field =
                    FieldRef::new($tag, $occurrence, $subfields);
                let result = parse_field_ref.parse($i).unwrap();
                assert_eq!(result, field);
            };
            ($i:expr, $tag:expr, $subfields:expr) => {
                let field = FieldRef::new($tag, None, $subfields);
                let result = parse_field_ref.parse($i).unwrap();
                assert_eq!(result, field);
            };
            ($i:expr, $tag:expr) => {
                let field = FieldRef::new($tag, None, vec![]);
                let result = parse_field_ref.parse($i).unwrap();
                assert_eq!(result, field);
            };
        }

        parse_success!(
            b"012A/01 \x1fabc\x1e",
            "012A",
            Some("01"),
            vec![('a', "bc")]
        );

        parse_success!(b"012A \x1fabc\x1e", "012A", vec![('a', "bc")]);
        parse_success!(b"012A \x1e", "012A");

        macro_rules! parse_error {
            ($i:expr) => {
                assert!(parse_field_ref.parse($i).is_err());
            };
        }

        parse_error!(b"012A/00\x1fabc\x1e");
        parse_error!(b"012A/00 abc\x1e");
        parse_error!(b"012A/00 \x1fabc");
        parse_error!(b"012!/01 \x1fabc\x1e");
        parse_error!(b"012A/0! \x1fabc\x1e");
        parse_error!(b"012A/00 \x1f!bc\x1e");
    }

    #[cfg_attr(miri, ignore)]
    #[quickcheck_macros::quickcheck]
    fn test_parse_arbitrary_field_ref(field: Field) -> bool {
        let mut bytes = Vec::<u8>::new();
        let _ = field.write_to(&mut bytes);

        parse_field_ref.parse(&bytes).is_ok()
    }
}
