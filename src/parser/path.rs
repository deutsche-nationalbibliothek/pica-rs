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
