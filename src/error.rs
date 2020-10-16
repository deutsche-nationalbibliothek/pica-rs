use std::{error::Error, fmt};

/// Error which will be returned if a record could not be parsed.
///
/// ```should_panic
/// use pica::Record;
/// let _rec: Record = "003@ \u{1f}0123456789"
///     .parse()
///     .expect("missing record separator.");
/// ```
#[derive(Debug, PartialEq)]
pub struct PicaParseError {}

impl fmt::Display for PicaParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid Pica+ record.")
    }
}

impl Error for PicaParseError {}
