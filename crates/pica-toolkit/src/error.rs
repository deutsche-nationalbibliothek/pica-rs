use std::io;

pub(crate) type CliResult<T> = Result<T, CliError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum CliError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Csv(#[from] csv::Error),

    #[error(transparent)]
    ParsePica(#[from] pica_record::ParsePicaError),

    #[error(transparent)]
    ReadPica(#[from] pica_record::io::ReadPicaError),

    #[error(transparent)]
    ParsePath(#[from] pica_path::ParsePathError),

    #[error(transparent)]
    ParseMatcher(#[from] pica_matcher::ParseMatcherError),

    #[error(transparent)]
    ParseQuery(#[from] pica_select::ParseQueryError),

    #[error("{0}")]
    Other(String),
}
