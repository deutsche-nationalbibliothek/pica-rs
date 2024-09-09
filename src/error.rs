use thiserror::Error;

// macro_rules! bail {
//     ($($tt:tt)*) => {{
//         return Err(crate::error::Error::Other(format!($($tt)*)))
//     }};
// }

// pub(crate) use bail;

/// An error that can occur in this library.
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ParsePica(ParsePicaError),

    #[error("{0}")]
    Other(String),
}

/// An error that can occur when parsing PICA+ records.
#[derive(Debug, Error)]
pub enum ParsePicaError {
    #[error("'{0}' is not a valid subfield code.")]
    SubfieldCode(char),
    #[error("'{0}' is not a valid subfield value.")]
    SubfieldValue(String),
    #[error("'{0}' is not a valid subfield.")]
    Subfield(String),
}
