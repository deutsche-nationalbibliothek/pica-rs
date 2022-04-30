/// Parser result.
pub type ParseResult<'a, O> = Result<(&'a [u8], O), nom::Err<()>>;
