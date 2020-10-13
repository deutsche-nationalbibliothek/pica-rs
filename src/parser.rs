use nom::{
    character::complete::{none_of, one_of},
    combinator::{map, opt, recognize},
    multi::{count, many1},
    sequence::{pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use std::iter::FromIterator;

#[derive(Debug, PartialEq, Eq)]
pub struct Subfield {
    pub code: char,
    pub value: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Field {
    pub tag: String,
    pub occurrence: Option<String>,
    pub subfields: Vec<Subfield>,
}

/// Parse a Pica+ tag.
///
/// A Pica+ tag starts with a digit less than three followed by two digits and
/// an uppercase letter or an '@' character. If the parser succeeds, the
/// remaining input and the tag is returned as an tuple wrapped in an [`Ok`].
///
/// # Example
/// ```
/// use pica::parser::parse_tag;
///
/// let (_, tag) = parse_tag("003@").unwrap();
/// assert_eq!(tag, "003@");
/// ```
pub fn parse_tag(i: &str) -> IResult<&str, String> {
    map(
        recognize(tuple((
            one_of("012"),
            count(one_of("0123456789"), 2),
            one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ@"),
        ))),
        |s: &str| String::from_iter(s.chars()),
    )(i)
}

/// Parse field occurrence.
///
/// The field occurrence is preceded by one '/' character followed by two
/// digits. If the parser succeeds, the remaining input and the occurrence is
/// returned as an tuple wrapped in an [`Ok`].
///
/// # Example
/// ```
/// use pica::parser::parse_occurrence;
///
/// let (_, occ) = parse_occurrence("/00").unwrap();
/// assert_eq!(occ, "00");
/// ```
pub fn parse_occurrence(i: &str) -> IResult<&str, String> {
    map(
        preceded(
            nom::character::complete::char('/'),
            recognize(count(one_of("0123456789"), 2)),
        ),
        |s: &str| String::from_iter(s.chars()),
    )(i)
}

/// Parse a subfield.
///
/// A subfield starts with the unit separator (\x1f) followed by the subfield
/// code (alpha numerical character). The optional subfield value ends with a
/// unit separator or an record separator (\x1f). If the parse succeeds the
/// remaining input and the parsed [`Subfield`] is returned as an tuple wrapped
/// in an [`Ok`].
///
/// # Example
///
/// ```
/// use pica::parser::parse_subfield;
///
/// let (_, subfield) = parse_subfield("\x1fa123456789").unwrap();
/// assert_eq!(subfield.code, 'a');
/// assert_eq!(subfield.value, Some("123456789".to_string()));
/// ```
pub fn parse_subfield(i: &str) -> IResult<&str, Subfield> {
    preceded(
        nom::character::complete::char('\x1f'),
        map(
            pair(
                one_of("a0bd94eA7VSEBHtgDhcfpnmYrK5iuLv6xGjlFqJIoTyzOMP2sRNUX3kZQCw18W"),
                opt(map(many1(none_of("\x1f\x1e")), |s| String::from_iter(s))),
            ),
            |(c, v)| Subfield { code: c, value: v },
        ),
    )(i)
}

/// Parse a field.
///
/// A field consists of an field tag, a non-empty list of subfields and ends
/// with an record separator (\x1e). If the parse succeeds the remaining input
/// and the parsed [`Field`] is returned as an tuple wrapped in an [`Ok`].
///
/// # Example
/// ```
/// use pica::parser::parse_field;
///
/// let (_, field) = parse_field("003@ \x1f0123456789\x1e").unwrap();
/// assert_eq!(field.tag, "003@");
/// ```
pub fn parse_field(i: &str) -> IResult<&str, Field> {
    terminated(
        map(
            separated_pair(
                pair(parse_tag, opt(parse_occurrence)),
                nom::character::complete::char(' '),
                many1(parse_subfield),
            ),
            |((tag, occurrence), subfields)| Field {
                tag,
                occurrence,
                subfields,
            },
        ),
        nom::character::complete::char('\x1e'),
    )(i)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Record {
    pub fields: Vec<Field>,
}

/// Parse reccord
///
/// # Example
/// ```
/// use pica::parser::parse_record;
///
/// let (_, record) = parse_record("003@ \x1f0123456789\x1e").unwrap();
/// assert_eq!(record.fields.len(), 1);
/// ```
pub fn parse_record(i: &str) -> IResult<&str, Record> {
    map(many1(parse_field), |fields| Record { fields })(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag() {
        assert_eq!(parse_tag("003@"), Ok(("", "003@".to_string())));
        assert!(parse_tag("300A").is_err());
        assert!(parse_tag("0A0A").is_err());
        assert!(parse_tag("00AA").is_err());
        assert!(parse_tag("0000").is_err());
    }

    #[test]
    fn test_parse_occurrence() {
        assert_eq!(parse_occurrence("/00"), Ok(("", "00".to_string())));
        assert_eq!(parse_occurrence("/01"), Ok(("", "01".to_string())));
        assert!(parse_occurrence("00").is_err());
    }

    #[test]
    fn test_parse_subfield() {
        assert_eq!(
            parse_subfield("\x1fa123456789"),
            Ok((
                "",
                Subfield {
                    code: 'a',
                    value: Some("123456789".to_string())
                }
            ))
        );

        assert_eq!(
            parse_subfield("\x1fa"),
            Ok((
                "",
                Subfield {
                    code: 'a',
                    value: None,
                }
            ))
        );
    }

    #[test]
    fn test_parse_field() {
        assert_eq!(
            parse_field("003@ \x1f0123456789\x1e"),
            Ok((
                "",
                Field {
                    tag: "003@".to_string(),
                    occurrence: None,
                    subfields: vec![Subfield {
                        code: '0',
                        value: Some("123456789".to_string())
                    }]
                }
            ))
        );

        assert_eq!(
            parse_field("001B/01 \x1f0123456789\x1e"),
            Ok((
                "",
                Field {
                    tag: "001B".to_string(),
                    occurrence: Some("01".to_string()),
                    subfields: vec![Subfield {
                        code: '0',
                        value: Some("123456789".to_string())
                    }]
                }
            ))
        );
    }

    #[test]
    fn test_parse_record() {
        assert_eq!(
            parse_record("003@ \x1f0123456789\x1e"),
            Ok((
                "",
                Record {
                    fields: vec![Field {
                        tag: "003@".to_string(),
                        occurrence: None,
                        subfields: vec![Subfield {
                            code: '0',
                            value: Some("123456789".to_string())
                        }]
                    }]
                }
            ))
        );
    }
}
