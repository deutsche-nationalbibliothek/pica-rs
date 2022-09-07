use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParsePicaError {
    #[error("invalid subfield")]
    InvalidSubfield,
}
