use crate::parser::{
    parse_field_occurrence, parse_field_tag, parse_subfield_code, ws,
};
use crate::Path;
use nom::character::complete::{char, digit1, multispace0};
use nom::combinator::{all_consuming, map, opt};
use nom::multi::separated_list1;
use nom::sequence::{delimited, preceded, tuple};
use nom::IResult;

pub fn parse_path(i: &str) -> IResult<&str, Path> {
    map(
        tuple((
            preceded(multispace0, parse_field_tag),
            opt(parse_field_occurrence),
            preceded(char('.'), parse_subfield_code),
            opt(delimited(
                char('['),
                map(digit1, |v: &str| v.parse::<usize>().unwrap()),
                char(']'),
            )),
            multispace0,
        )),
        |(tag, occurrence, code, index, _)| {
            Path::new(tag, occurrence, code, index)
        },
    )(i)
}

pub fn parse_path_list(i: &str) -> IResult<&str, Vec<Path>> {
    all_consuming(separated_list1(char(','), ws(parse_path)))(i)
}

#[cfg(config)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path() {
        let data: Vec<(&str, Path)> = vec![
            ("003@.0", Path::new("003@", None, '0', None)),
            ("  003@.0 ", Path::new("003@", None, '0', None)),
            ("012A/01.0", Path::new("012A", Some("01"), '0', None)),
            ("003@.0[0]", Path::new("003@", None, '0', Some(0))),
            ("012A/00.0[1]", Path::new("012A", Some("00"), '0', Some(1))),
        ];

        for (input, expected) in data {
            assert_eq!(parse_path(input), Ok(("", expected)));
        }
    }

    #[test]
    fn test_parse_path_list() {
        let path_list =
            vec![Path::from_str("003@.0"), Path::from_str("002@.0[0]")];

        assert_eq!(
            parse_path_list(" 003@.0, 002@.0[0]  "),
            Ok(("", path_list))
        );
    }
}
