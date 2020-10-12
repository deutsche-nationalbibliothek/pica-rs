use nom::IResult;

use nom::{
    character::complete::one_of,
    combinator::recognize,
    multi::count,
    sequence::{preceded, tuple},
};

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
}
