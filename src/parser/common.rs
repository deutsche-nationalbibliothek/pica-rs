//! Helper functions

use nom::character::complete::multispace0;
use nom::error::ParseError;
use nom::sequence::delimited;
use nom::Parser;

pub fn ws<'a, O, E: ParseError<&'a str>, F: Parser<&'a str, O, E>>(
    f: F,
) -> impl Parser<&'a str, O, E> {
    delimited(multispace0, f, multispace0)
}
