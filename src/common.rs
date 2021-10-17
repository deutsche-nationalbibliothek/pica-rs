/// Common parser types and functions.
use nom::Err;

/// Parser result.
pub(crate) type ParseResult<'a, O> = Result<(&'a [u8], O), Err<()>>;
