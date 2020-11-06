use crate::parser::{
    parse_field_occurrence, parse_field_tag, parse_subfield_code,
};
use crate::Path;
use nom::character::complete::space0;
use nom::combinator::{map, opt};
use nom::sequence::{delimited, pair, separated_pair};
use nom::IResult;

pub fn parse_path(i: &str) -> IResult<&str, Path> {
    map(
        delimited(
            space0,
            separated_pair(
                pair(parse_field_tag, opt(parse_field_occurrence)),
                nom::character::complete::char('.'),
                parse_subfield_code,
            ),
            space0,
        ),
        |((tag, occurrence), code)| {
            Path::new(tag, occurrence.unwrap_or_default(), code)
        },
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path() {
        assert_eq!(
            parse_path("   003@.0 "),
            Ok(("", Path::new("003@", "", '0')))
        );
        assert_eq!(
            parse_path("   003@/00.0 "),
            Ok(("", Path::new("003@", "00", '0')))
        );
    }
}
