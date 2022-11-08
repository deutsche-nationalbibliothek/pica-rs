use nom::character::complete::multispace0;
use nom::sequence::delimited;
use nom::IResult;

/// Strip whitespaces from the beginning and end.
pub(crate) fn ws<'a, F: 'a, O, E: nom::error::ParseError<&'a [u8]>>(
    inner: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], O, E>,
{
    delimited(multispace0, inner, multispace0)
}
