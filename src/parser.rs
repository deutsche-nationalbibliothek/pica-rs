use crate::Subfield;
use nom::branch::alt;
use nom::character::complete::{char, none_of, one_of};
use nom::combinator::map;
use nom::combinator::recognize;
use nom::multi::many0;
use nom::sequence::{pair, preceded};
use nom::IResult;

pub(crate) fn parse_subfield_code(i: &str) -> IResult<&str, char> {
    alt((
        one_of("abcdefghijklmnopqrstuvwxyz"),
        one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        one_of("0123456789"),
    ))(i)
}

pub(crate) fn parse_subfield_value(i: &str) -> IResult<&str, &str> {
    recognize(many0(none_of("\u{1e}\u{1f}")))(i)
}

pub fn parse_subfield(i: &str) -> IResult<&str, Subfield> {
    preceded(
        char('\u{1f}'),
        map(
            pair(parse_subfield_code, parse_subfield_value),
            |(code, value)| Subfield::from_unchecked(code, value),
        ),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_subfield_code() {
        for c in 'a'..='z' {
            assert_eq!(parse_subfield_code(&String::from(c)), Ok(("", c)));
        }
        for c in 'A'..='Z' {
            assert_eq!(parse_subfield_code(&String::from(c)), Ok(("", c)));
        }
        for c in '0'..='9' {
            assert_eq!(parse_subfield_code(&String::from(c)), Ok(("", c)));
        }
    }

    #[test]
    fn test_parse_subfield_value() {
        assert_eq!(parse_subfield_value(""), Ok(("", "")));
        assert_eq!(parse_subfield_value("abc"), Ok(("", "abc")));
        assert_eq!(parse_subfield_value("ab\u{1f}c"), Ok(("\u{1f}c", "ab")));
        assert_eq!(parse_subfield_value("ab\u{1e}c"), Ok(("\u{1e}c", "ab")));
    }

    #[test]
    fn test_parse_subfield() {
        assert_eq!(
            parse_subfield("\u{1f}a123"),
            Ok((
                "",
                Subfield {
                    code: 'a',
                    value: "123".to_string()
                }
            ))
        );
        assert!(parse_subfield("!a123").is_err());
    }
}
