use crate::primitives::ParsePicaError;

/// An error that can occur in this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ParsePica(ParsePicaError),
}
