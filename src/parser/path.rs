use crate::parser::{
    parse_field_occurrence, parse_field_tag, parse_subfield_code, ws,
};
use crate::Path;
use nom::character::complete::{char, digit1};
use nom::combinator::{map, opt};
use nom::sequence::{delimited, preceded, tuple};
use nom::IResult;

pub fn parse_path(i: &str) -> IResult<&str, Path> {
    map(
        ws(tuple((
            parse_field_tag,
            opt(parse_field_occurrence),
            preceded(char('.'), parse_subfield_code),
            opt(delimited(
                char('['),
                map(digit1, |v: &str| v.parse::<usize>().unwrap()),
                char(']'),
            )),
        ))),
        |(tag, occurrence, code, index)| {
            Path::new(tag, occurrence, code, index)
        },
    )(i)
}
