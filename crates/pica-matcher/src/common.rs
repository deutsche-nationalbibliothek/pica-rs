use std::fmt::{self, Display};

use winnow::ascii::{multispace0, multispace1};
use winnow::combinator::{
    alt, delimited, preceded, repeat, terminated,
};
use winnow::error::{ContextError, ParserError};
use winnow::stream::{AsChar, Compare, Stream, StreamIsPartial};
use winnow::token::take_till;
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
pub enum RelationalOp {
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

impl RelationalOp {
    /// Returns true of the operator can be used in combination with the
    /// `usize` type, otherwise false.
    pub fn is_usize_applicable(&self) -> bool {
        matches!(
            self,
            RelationalOp::Eq
                | RelationalOp::Ne
                | RelationalOp::Ge
                | RelationalOp::Gt
                | RelationalOp::Lt
                | RelationalOp::Le
        )
    }

    /// Returns true of the operator can be used in combination with
    /// `str` or byte slices, otherwise false.
    pub fn is_str_applicable(&self) -> bool {
        !matches!(
            self,
            RelationalOp::Ge
                | RelationalOp::Gt
                | RelationalOp::Lt
                | RelationalOp::Le
        )
    }
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
        "==".value(RelationalOp::Eq),
        "!=".value(RelationalOp::Ne),
        "=^".value(RelationalOp::StartsWith),
        "!^".value(RelationalOp::StartsNotWith),
        "=$".value(RelationalOp::EndsWith),
        "!$".value(RelationalOp::EndsNotWith),
        "=*".value(RelationalOp::Similar),
        "=?".value(RelationalOp::Contains),
    ))
    .parse_next(i)
}

/// Parse RelationalOp which can be used for usize comparisons.
#[inline]
pub(crate) fn parse_relational_op_usize(
    i: &mut &[u8],
) -> PResult<RelationalOp> {
    alt((
        "==".value(RelationalOp::Eq),
        "!=".value(RelationalOp::Ne),
        ">=".value(RelationalOp::Ge),
        ">".value(RelationalOp::Gt),
        "<=".value(RelationalOp::Le),
        "<".value(RelationalOp::Lt),
    ))
    .parse_next(i)
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Quantifier {
    All,
    #[default]
    Any,
}

#[inline]
pub(crate) fn parse_quantifier(i: &mut &[u8]) -> PResult<Quantifier> {
    alt((
        terminated("ALL".value(Quantifier::All), multispace1),
        terminated("ANY".value(Quantifier::Any), multispace1),
        "∀".value(Quantifier::All),
        "∃".value(Quantifier::Any),
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
        Quotes::Single => take_till(1.., ['\'', '\\']),
        Quotes::Double => take_till(1.., ['"', '\\']),
    }
}

fn parse_escaped_char<I, E>(quotes: Quotes) -> impl Parser<I, char, E>
where
    I: Stream + StreamIsPartial + Compare<char>,
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
    Literal(&'a [u8]),
    EscapedChar(char),
    EscapedWs,
}

fn parse_quoted_fragment<'a, E: ParserError<&'a [u8]>>(
    quotes: Quotes,
) -> impl Parser<&'a [u8], StringFragment<'a>, E> {
    use StringFragment::*;

    alt((
        parse_literal::<&'a [u8], E>(quotes).map(Literal),
        parse_escaped_char::<&'a [u8], E>(quotes).map(EscapedChar),
        preceded('\\', multispace1).value(EscapedWs),
    ))
}

fn parse_quoted_string<'a, E>(
    quotes: Quotes,
) -> impl Parser<&'a [u8], Vec<u8>, E>
where
    E: ParserError<&'a [u8]>,
{
    use StringFragment::*;

    let string_builder = repeat(
        0..,
        parse_quoted_fragment::<E>(quotes),
    )
    .fold(Vec::new, |mut acc, fragment| {
        match fragment {
            Literal(s) => acc.extend_from_slice(s),
            EscapedChar(c) => acc.push(c as u8),
            EscapedWs => {}
        }
        acc
    });

    match quotes {
        Quotes::Single => delimited('\'', string_builder, '\''),
        Quotes::Double => delimited('"', string_builder, '"'),
    }
}

#[inline]
fn parse_string_single_quoted(i: &mut &[u8]) -> PResult<Vec<u8>> {
    parse_quoted_string::<ContextError>(Quotes::Single).parse_next(i)
}

#[inline]
fn parse_string_double_quoted(i: &mut &[u8]) -> PResult<Vec<u8>> {
    parse_quoted_string::<ContextError>(Quotes::Double).parse_next(i)
}

pub(crate) fn parse_string(i: &mut &[u8]) -> PResult<Vec<u8>> {
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

        parse_success!(b"'abc'", b"abc");
        parse_success!(b"'a\"bc'", b"a\"bc");
        parse_success!(b"'a\\'bc'", b"a'bc");
        parse_success!(b"''", b"");
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

        parse_success!(b"\"abc\"", b"abc");
        parse_success!(b"\"a\\\"bc\"", b"a\"bc");
        parse_success!(b"\"a\'bc\"", b"a'bc");
        parse_success!(b"\"\"", b"");
    }
}
