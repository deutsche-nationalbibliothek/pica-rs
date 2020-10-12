use nom::{
    branch::alt,
    character::complete::{none_of, one_of},
    combinator::{map, opt, recognize},
    multi::{count, many1},
    sequence::{pair, preceded, tuple},
    IResult,
};
use std::iter::FromIterator;

#[derive(Debug, PartialEq, Eq)]
pub struct Subfield {
    pub code: char,
    pub value: Option<String>,
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
pub fn parse_tag(i: &str) -> IResult<&str, &str> {
    recognize(tuple((
        one_of("012"),
        count(one_of("0123456789"), 2),
        one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ@"),
    )))(i)
}

/// Parse the field occurrence.
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
pub fn parse_occurrence(i: &str) -> IResult<&str, &str> {
    preceded(
        nom::character::complete::char('/'),
        recognize(count(one_of("0123456789"), 2)),
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
                alt((
                    one_of("ABCdefghijklmnopqrstuvwxyz"),
                    one_of("abcdefghijklmnopqrstuvwxyz"),
                    one_of("0123456789"),
                )),
                opt(map(many1(none_of("\x1f\x1e")), |s| String::from_iter(s))),
            ),
            |(c, v)| Subfield { code: c, value: v },
        ),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag() {
        assert_eq!(parse_tag("003@"), Ok(("", "003@")));
        assert!(parse_tag("300A").is_err());
        assert!(parse_tag("0A0A").is_err());
        assert!(parse_tag("00AA").is_err());
        assert!(parse_tag("0000").is_err());
    }

    #[test]
    fn test_parse_occurrence() {
        assert_eq!(parse_occurrence("/00"), Ok(("", "00")));
        assert_eq!(parse_occurrence("/01"), Ok(("", "01")));
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
}
