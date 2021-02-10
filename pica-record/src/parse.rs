use bstr::{BStr, ByteSlice};

use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{one_of, satisfy};
use nom::combinator::{all_consuming, cut, map, opt, recognize};
use nom::multi::{count, many0, many1, many_m_n};
use nom::sequence::{pair, preceded, terminated, tuple};

use crate::{Field, Record, Subfield};

pub type ParseResult<'a, O> = Result<(&'a [u8], O), nom::Err<()>>;

const NL: &[u8; 1] = b"\x0A";
const RS: &[u8; 1] = b"\x1E";
const SP: &[u8; 1] = b"\x20";
const US: &[u8; 1] = b"\x1F";

pub fn subfield_name(i: &[u8]) -> ParseResult<char> {
    map(satisfy(|c| c.is_ascii_alphanumeric()), char::from)(i)
}

pub fn subfield_value(i: &[u8]) -> ParseResult<&BStr> {
    recognize(many0(is_not("\x1E\x1F")))(i).map(|(i, o)| (i, o.as_bstr()))
}

pub fn subfield(i: &[u8]) -> ParseResult<Subfield> {
    map(
        preceded(tag(US), cut(pair(subfield_name, subfield_value))),
        |(name, value)| Subfield {
            name,
            value: value.as_bstr(),
        },
    )(i)
}

pub fn field_name(i: &[u8]) -> ParseResult<&BStr> {
    map(
        recognize(tuple((
            one_of("012"),
            count(one_of("0123456789"), 2),
            one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ@"),
        ))),
        |value: &[u8]| value.as_bstr(),
    )(i)
}

pub fn field_occurrence(i: &[u8]) -> ParseResult<&BStr> {
    map(
        preceded(
            tag(b"/"),
            cut(recognize(many_m_n(2, 3, one_of("0123456789")))),
        ),
        |value: &[u8]| value.as_bstr(),
    )(i)
}

pub fn field(i: &[u8]) -> ParseResult<Field> {
    let (i, ((name, occurrence), subfields)) = terminated(
        pair(
            pair(field_name, opt(field_occurrence)),
            preceded(tag(SP), many0(subfield)),
        ),
        tag(RS),
    )(i)?;

    let field = Field {
        name,
        occurrence,
        subfields,
    };

    Ok((i, field))
}

pub fn record(i: &[u8]) -> ParseResult<Record> {
    map(
        all_consuming(terminated(many1(field), opt(tag(NL)))),
        |fields| Record(fields),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_subfield() {
        assert_eq!(
            subfield(b"\x1f0123456789X").expect("parse subfield").1,
            Subfield {
                name: '0',
                value: b"123456789X".as_bstr()
            }
        )
    }

    #[test]
    fn parse_field() {
        assert_eq!(
            field(b"003@ \x1f0123456789X\x1E").expect("parse field").1,
            Field {
                name: b"003@".as_bstr(),
                occurrence: None,
                subfields: vec![Subfield {
                    name: '0',
                    value: b"123456789X".as_bstr()
                }]
            }
        );

        assert_eq!(
            field(b"012A/000 \x1f0123456789X\x1E")
                .expect("parse field")
                .1,
            Field {
                name: b"012A".as_bstr(),
                occurrence: Some(b"000".as_bstr()),
                subfields: vec![Subfield {
                    name: '0',
                    value: b"123456789X".as_bstr()
                }]
            }
        )
    }

    #[test]
    fn parse_record() {
        assert_eq!(
            record(b"003@ \x1f0123456789X\x1E012A/00 \x1fa123\x1e")
                .expect("parse record")
                .1,
            Record(vec![
                Field {
                    name: b"003@".as_bstr(),
                    occurrence: None,
                    subfields: vec![Subfield {
                        name: '0',
                        value: b"123456789X".as_bstr()
                    }]
                },
                Field {
                    name: b"012A".as_bstr(),
                    occurrence: Some(b"00".as_bstr()),
                    subfields: vec![Subfield {
                        name: 'a',
                        value: b"123".as_bstr()
                    }]
                },
            ])
        );
    }
}
