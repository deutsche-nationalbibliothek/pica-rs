use std::fmt::{self, Display};

use winnow::combinator::alt;
use winnow::error::ParserError;
use winnow::stream::Stream;
use winnow::token::{tag, take_until1};
use winnow::{PResult, Parser};

// use nom::branch::alt;
// use nom::bytes::complete::{is_not, tag};
// use nom::character::complete::{char, multispace0, multispace1};
// use nom::combinator::{map, map_res, value, verify};
// use nom::multi::fold_many0;
// use nom::sequence::{delimited, preceded};
// use nom::IResult;
// use pica_record::parser::ParseResult;

// /// Boolean Operators.
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum BooleanOp {
//     And, // and, "&&"
//     Or,  // or, "||"
// }

// /// Strip whitespaces from the beginning and end.
// pub(crate) fn ws<'a, F: 'a, O, E: nom::error::ParseError<&'a [u8]>>(
//     inner: F,
// ) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>
// where
//     F: Fn(&'a [u8]) -> IResult<&'a [u8], O, E>,
// {
//     delimited(multispace0, inner, multispace0)
// }

/// Relational Operator
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RelationalOp {
    Eq,            // equal, "=="
    Ne,            // not equal, "!="
    Gt,            // greater than, ">"
    Ge,            // greater than or equal, ">="
    Lt,            // less than, "<"
    Le,            // less than or equal, "<="
    StartsWith,    // starts with, "=^"
    StartsNotWith, // starts not with, "!^"
    EndsWith,      // ends with, "=$"
    EndsNotWith,   // ends not with, "!$"
    Similar,       // similar, "=*"
    Contains,      // contains, "=?"
}

impl Display for RelationalOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            RelationalOp::Eq => write!(f, "=="),
            RelationalOp::Ne => write!(f, "!="),
            RelationalOp::Gt => write!(f, ">"),
            RelationalOp::Ge => write!(f, ">="),
            RelationalOp::Lt => write!(f, "<"),
            RelationalOp::Le => write!(f, "<="),
            RelationalOp::StartsWith => write!(f, "=^"),
            RelationalOp::StartsNotWith => write!(f, "!^"),
            RelationalOp::EndsWith => write!(f, "=$"),
            RelationalOp::EndsNotWith => write!(f, "!$"),
            RelationalOp::Similar => write!(f, "=*"),
            RelationalOp::Contains => write!(f, "=?"),
        }
    }
}

/// Parse RelationalOp which can be used for string comparisons.
#[inline]
pub(crate) fn parse_relational_op_str(
    i: &mut &[u8],
) -> PResult<RelationalOp> {
    alt((
        tag("==").value(RelationalOp::Eq),
        tag("!=").value(RelationalOp::Ne),
        tag("=^").value(RelationalOp::StartsWith),
        tag("!^").value(RelationalOp::StartsNotWith),
        tag("=$").value(RelationalOp::EndsWith),
        tag("!$").value(RelationalOp::EndsNotWith),
        tag("=*").value(RelationalOp::Similar),
        tag("=?").value(RelationalOp::Contains),
    ))
    .parse_next(i)
}

/// Parse RelationalOp which can be used for usize comparisons.
#[inline]
pub(crate) fn parse_relational_op_usize(
    i: &mut &[u8],
) -> PResult<RelationalOp> {
    alt((
        tag("==").value(RelationalOp::Eq),
        tag("!=").value(RelationalOp::Ne),
        tag(">=").value(RelationalOp::Ge),
        tag(">").value(RelationalOp::Gt),
        tag("<=").value(RelationalOp::Le),
        tag("<").value(RelationalOp::Lt),
    ))
    .parse_next(i)
}

#[derive(Debug, Copy, Clone)]
enum Quotes {
    Single,
    Double,
}

// #[derive(Debug, Clone)]
// enum StringFragment<'a> {
//     Literal(&'a str),
//     EscapedChar(char),
//     EscapedWs,
// }

/// Parse a non-empty block of text that doesn't include \ or ".
#[inline]
fn parse_literal<'a, I: Stream, E: ParserError<I>>(
    quotes: Quotes,
    i: I,
) -> impl Parser<I, &str, E> {
    move |i: &mut I| match quotes {
        Quotes::Single => take_until1("\'\\")
            .try_map(std::str::from_utf8)
            .parse_next(i),
        Quotes::Double => take_until1("\"\\")
            .try_map(std::str::from_utf8)
            .parse_next(i),
    }
}

// /// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
// fn parse_escaped_char(
//     quotes: Quotes,
// ) -> impl Fn(&[u8]) -> ParseResult<char> { move |i: &[u8]| { let val
//   = match quotes { Quotes::Single => '"', Quotes::Double => '\'', };

//         preceded(
//             char('\\'),
//             alt((
//                 // parse_unicode,
//                 value('\n', char('n')),
//                 value('\r', char('r')),
//                 value('\t', char('t')),
//                 value('\u{08}', char('b')),
//                 value('\u{0C}', char('f')),
//                 value('\\', char('\\')),
//                 value('/', char('/')),
//                 value(val, char(val)),
//             )),
//         )(i)
//     }
// }

// /// Combine parse_literal, parse_escaped_char into a StringFragment.
// fn parse_fragment(
//     quotes: Quotes,
// ) -> impl Fn(&[u8]) -> ParseResult<StringFragment> { move |i: &[u8]|
//   { alt(( map(parse_literal(quotes), StringFragment::Literal), map(
//   parse_escaped_char(quotes), StringFragment::EscapedChar, ), value(
//   StringFragment::EscapedWs, preceded(char('\\'), multispace1), ),
//   ))(i) }
// }

// fn parse_string_inner(
//     quotes: Quotes,
// ) -> impl Fn(&[u8]) -> ParseResult<String> { move |i: &[u8]| {
//   fold_many0( parse_fragment(quotes), String::new, |mut string,
//   fragment| { match fragment { StringFragment::Literal(s) =>
//   string.push_str(s), StringFragment::EscapedChar(c) =>
//   string.push(c), StringFragment::EscapedWs => {} } string }, )(i) }
// }

// fn parse_string_single_quoted(i: &[u8]) -> ParseResult<String> {
//     delimited(
//         char('\''),
//         parse_string_inner(Quotes::Single),
//         char('\''),
//     )(i)
// }

// fn parse_string_double_quoted(i: &[u8]) -> ParseResult<String> {
//     delimited(char('"'), parse_string_inner(Quotes::Double),
// char('"'))(         i,
//     )
// }

// pub(crate) fn parse_string(i: &[u8]) -> ParseResult<String> {
//     alt((parse_string_single_quoted, parse_string_double_quoted))(i)
// }

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_relational_op_str() {
        use super::parse_relational_op_str;

        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    parse_relational_op_str
                        .parse($input.as_bytes())
                        .unwrap(),
                    $expected
                );
            };
        }

        parse_success!("==", RelationalOp::Eq);
        parse_success!("!=", RelationalOp::Ne);
        parse_success!("=^", RelationalOp::StartsWith);
        parse_success!("!^", RelationalOp::StartsNotWith);
        parse_success!("=$", RelationalOp::EndsWith);
        parse_success!("!$", RelationalOp::EndsNotWith);
        parse_success!("=*", RelationalOp::Similar);
        parse_success!("=?", RelationalOp::Contains);

        assert!(parse_relational_op_str.parse(b"=>").is_err());
        assert!(parse_relational_op_str.parse(b">").is_err());
        assert!(parse_relational_op_str.parse(b"<").is_err());
        assert!(parse_relational_op_str.parse(b"<=").is_err());
    }

    #[test]
    fn parse_relational_op_usize() {
        use super::parse_relational_op_usize;

        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    parse_relational_op_usize
                        .parse($input.as_bytes())
                        .unwrap(),
                    $expected
                );
            };
        }

        parse_success!("==", RelationalOp::Eq);
        parse_success!("!=", RelationalOp::Ne);
        parse_success!(">=", RelationalOp::Ge);
        parse_success!(">", RelationalOp::Gt);
        parse_success!("<=", RelationalOp::Le);
        parse_success!("<", RelationalOp::Lt);

        assert!(parse_relational_op_usize.parse(b"=*").is_err());
        assert!(parse_relational_op_usize.parse(b"=~").is_err());
        assert!(parse_relational_op_usize.parse(b"=^").is_err());
        assert!(parse_relational_op_usize.parse(b"!^").is_err());
        assert!(parse_relational_op_usize.parse(b"=$").is_err());
        assert!(parse_relational_op_usize.parse(b"!$").is_err());
        assert!(parse_relational_op_usize.parse(b"=?").is_err());
    }

    #[test]
    fn parse_literal() {
        use super::parse_literal;
        todo!()
    }

    //     #[test]
    //     fn test_parse_string() {
    //         assert_done_and_eq!(parse_string(b"'abc'"),
    // "abc".to_string());
    // assert_done_and_eq!(parse_string(b"'\tc'"
    // ), "\tc".to_string());         assert_done_and_eq!(
    //             parse_string(b"\"abc\""),
    //             "abc".to_string()
    //         );
    //         assert_done_and_eq!(
    //             parse_string(b"\"\tc\""),
    //             "\tc".to_string()
    //         );
    //     }
}
