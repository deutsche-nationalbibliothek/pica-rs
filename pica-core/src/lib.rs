/// Holds the result of a parsing function.
///
/// It takes a byte slice as input and uses `nom::Err<()>` as error variant. The
/// type only depends the output type `O`.
pub type ParseResult<'a, O> = Result<(&'a [u8], O), nom::Err<()>>;
