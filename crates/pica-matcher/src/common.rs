use std::fmt::{self, Display};

use winnow::ascii::{multispace0, multispace1};
use winnow::combinator::{alt, delimited, fold_repeat, preceded};
use winnow::error::{ContextError, ParserError};
use winnow::stream::{AsChar, Stream, StreamIsPartial};
use winnow::token::{tag, take_till1};
use winnow::{PResult, Parser};

/// Boolean Operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BooleanOp {
    And, // and, "&&"
    Or,  // or, "||"
}

/// Strip whitespaces from the beginning and end.
pub(crate) fn ws<I, O, E: ParserError<I>, F>(
    mut inner: F,
) -> impl Parser<I, O, E>
where
    I: Stream + StreamIsPartial,
    <I as Stream>::Token: AsChar + Clone,
    F: Parser<I, O, E>,
{
    move |i: &mut I| {
        let _ = multispace0.parse_next(i)?;
        let o = inner.parse_next(i);
        let _ = multispace0.parse_next(i)?;

        o
    }
}

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

fn parse_literal<I, E>(
    quotes: Quotes,
) -> impl Parser<I, <I as Stream>::Slice, E>
where
    I: Stream + StreamIsPartial,
    <I as Stream>::Token: AsChar,
    E: ParserError<I>,
{
    match quotes {
        Quotes::Single => take_till1(['\'', '\\']),
        Quotes::Double => take_till1(['"', '\\']),
    }
}

fn parse_escaped_char<I, E>(quotes: Quotes) -> impl Parser<I, char, E>
where
    I: Stream + StreamIsPartial,
    <I as Stream>::Token: AsChar + Clone,
    E: ParserError<I>,
{
    let v = match quotes {
        Quotes::Single => '\'',
        Quotes::Double => '"',
    };

    preceded(
        '\\',
        alt((
            'n'.value('\n'),
            'r'.value('\r'),
            't'.value('\t'),
            'b'.value('\u{08}'),
            'f'.value('\u{0C}'),
            '\\'.value('\\'),
            '/'.value('/'),
            v.value(v),
        )),
    )
}

#[derive(Debug, Clone)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWs,
}

fn parse_quoted_fragment<'a, E: ParserError<&'a str>>(
    quotes: Quotes,
) -> impl Parser<&'a str, StringFragment<'a>, E> {
    use StringFragment::*;

    alt((
        parse_literal::<&'a str, E>(quotes).map(Literal),
        parse_escaped_char::<&'a str, E>(quotes).map(EscapedChar),
        preceded('\\', multispace1).value(EscapedWs),
    ))
}

fn parse_quoted_string<'a, E>(
    quotes: Quotes,
) -> impl Parser<&'a str, String, E>
where
    E: ParserError<&'a str>,
{
    use StringFragment::*;

    let string_builder = fold_repeat(
        0..,
        parse_quoted_fragment::<E>(quotes),
        String::new,
        |mut string, fragment| {
            match fragment {
                Literal(s) => string.push_str(s),
                EscapedChar(c) => string.push(c),
                EscapedWs => {}
            }
            string
        },
    );

    match quotes {
        Quotes::Single => delimited('\'', string_builder, '\''),
        Quotes::Double => delimited('"', string_builder, '"'),
    }
}

#[inline]
fn parse_string_single_quoted(i: &mut &str) -> PResult<String> {
    parse_quoted_string::<ContextError>(Quotes::Single).parse_next(i)
}

#[inline]
fn parse_string_double_quoted(i: &mut &str) -> PResult<String> {
    parse_quoted_string::<ContextError>(Quotes::Double).parse_next(i)
}

pub(crate) fn parse_string(i: &mut &str) -> PResult<String> {
    alt((parse_string_single_quoted, parse_string_double_quoted))
        .parse_next(i)
}

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
    fn parse_string_single_quoted() {
        use super::parse_string_single_quoted;

        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    parse_string_single_quoted.parse($input).unwrap(),
                    $expected
                );
            };
        }

        parse_success!("'abc'", "abc");
        parse_success!("'a\"bc'", "a\"bc");
        parse_success!("'a\\'bc'", "a'bc");
        parse_success!("''", "");
    }

    #[test]
    fn parse_string_double_quoted() {
        use super::parse_string_double_quoted;

        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    parse_string_double_quoted.parse($input).unwrap(),
                    $expected
                );
            };
        }

        parse_success!("\"abc\"", "abc");
        parse_success!("\"a\\\"bc\"", "a\"bc");
        parse_success!("\"a\'bc\"", "a'bc");
        parse_success!("\"\"", "");
    }
}
