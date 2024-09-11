use winnow::combinator::{opt, preceded, repeat, terminated};
use winnow::prelude::*;
use winnow::stream::AsChar;
use winnow::token::{one_of, take_till, take_while};

use super::{
    FieldRef, OccurrenceRef, RecordRef, SubfieldCode, SubfieldRef,
    SubfieldValueRef, TagRef,
};

/// Parses a [SubfieldCode] from a byte slice.
#[inline]
pub(crate) fn parse_subfield_code(
    i: &mut &[u8],
) -> PResult<SubfieldCode> {
    one_of((b'0'..=b'9', b'a'..=b'z', b'A'..=b'Z'))
        .map(SubfieldCode::from_unchecked)
        .parse_next(i)
}

/// Parse a PICA+ subfield value reference.
#[inline]
pub(crate) fn parse_subfield_value_ref<'a>(
    i: &mut &'a [u8],
) -> PResult<SubfieldValueRef<'a>> {
    take_till(0.., |c| c == b'\x1f' || c == b'\x1e')
        .map(SubfieldValueRef::from_unchecked)
        .parse_next(i)
}

/// Parse a PICA+ subfield.
pub(crate) fn parse_subfield_ref<'a>(
    i: &mut &'a [u8],
) -> PResult<SubfieldRef<'a>> {
    preceded(b'\x1f', (parse_subfield_code, parse_subfield_value_ref))
        .map(|(code, value)| SubfieldRef(code, value))
        .parse_next(i)
}

/// Parse a PICA+ tag.
pub(crate) fn parse_tag_ref<'a>(
    i: &mut &'a [u8],
) -> PResult<TagRef<'a>> {
    (
        one_of([b'0', b'1', b'2']),
        one_of(|c: u8| c.is_ascii_digit()),
        one_of(|c: u8| c.is_ascii_digit()),
        one_of(|c: u8| c.is_ascii_uppercase() || c == b'@'),
    )
        .take()
        .map(TagRef::from_unchecked)
        .parse_next(i)
}

/// Parse PICA+ occurrence occurrence.
#[inline]
pub(crate) fn parse_occurrence_ref<'a>(
    i: &mut &'a [u8],
) -> PResult<OccurrenceRef<'a>> {
    take_while(2..=3, AsChar::is_dec_digit)
        .map(OccurrenceRef::from_unchecked)
        .parse_next(i)
}

/// Parse a PICA+ field.
pub(crate) fn parse_field_ref<'a>(
    i: &mut &'a [u8],
) -> PResult<FieldRef<'a>> {
    (
        parse_tag_ref,
        opt(preceded(b'/', parse_occurrence_ref)),
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

/// Parse a [RecordRef].
#[inline]
pub(crate) fn parse_record_ref<'a>(
    i: &mut &'a [u8],
) -> PResult<RecordRef<'a>> {
    terminated(repeat(1.., parse_field_ref), b'\n')
        .map(RecordRef)
        .parse_next(i)
}

#[cfg(test)]
mod tests {
    use bstr::{ByteSlice, ByteVec};
    use quickcheck_macros::quickcheck;

    use super::*;
    use crate::primitives::{
        Field, Occurrence, Record, Subfield, SubfieldValue, Tag,
    };

    #[test]
    fn test_parse_subfield_code() {
        (u8::MIN..=u8::MAX).into_iter().for_each(|code| {
            if !code.is_ascii_alphanumeric() {
                assert!(parse_subfield_code.parse(&[code]).is_err());
            } else {
                assert_eq!(
                    parse_subfield_code.parse(&[code]).unwrap(),
                    SubfieldCode(code as char),
                )
            }
        });
    }

    #[quickcheck]
    fn test_parse_arbitrary_subfield_code(code: SubfieldCode) {
        assert_eq!(
            parse_subfield_code.parse(&[code.as_byte()]).unwrap(),
            code,
        )
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
    #[cfg_attr(miri, ignore)]
    fn test_parse_arbitrary_subfield_value_ref(value: SubfieldValue) {
        assert_eq!(
            parse_subfield_value_ref.parse(value.as_bytes()).unwrap(),
            value,
        )
    }

    #[quickcheck]
    #[cfg_attr(miri, ignore)]
    fn test_parse_arbitrary_subfield_ref(subfield: Subfield) {
        let mut bytes = Vec::new();
        subfield.write_to(&mut bytes).unwrap();

        let result = parse_subfield_ref.parse(&bytes).unwrap();
        assert_eq!(result.value(), subfield.value());
        assert_eq!(result.code(), subfield.code());
    }

    #[quickcheck]
    fn test_parse_arbitrary_tag_ref(tag: Tag) {
        let bytes = Vec::from_slice(tag.as_bytes());
        assert_eq!(parse_tag_ref.parse(&bytes).unwrap(), tag);
    }

    #[quickcheck]
    fn test_parse_arbitrary_occurrence_ref(occurrence: Occurrence) {
        let bytes = Vec::from_slice(occurrence.as_bytes());
        assert_eq!(
            parse_occurrence_ref.parse(&bytes).unwrap(),
            occurrence
        );
    }

    #[quickcheck]
    #[cfg_attr(miri, ignore)]
    fn test_parse_arbitrary_field_ref(field: Field) {
        let mut bytes = Vec::new();
        let _ = field.write_to(&mut bytes);

        assert_eq!(parse_field_ref.parse(&bytes).unwrap(), field);
    }

    #[quickcheck]
    #[cfg_attr(miri, ignore)]
    fn test_parse_arbitrary_record_ref(record: Record) {
        let mut bytes = Vec::new();
        let _ = record.write_to(&mut bytes);

        assert_eq!(parse_record_ref.parse(&bytes).unwrap(), record);
    }
}
