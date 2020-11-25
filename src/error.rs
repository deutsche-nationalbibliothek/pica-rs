use nom::error::{ErrorKind, ParseError};

#[derive(Debug, PartialEq)]
pub enum PicaParseError<I> {
    InvalidSubfiedCode(I),
    InvalidSubfieldValue(I),
    Nom(I, ErrorKind),
}

impl<I> ParseError<I> for PicaParseError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self::Nom(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}
