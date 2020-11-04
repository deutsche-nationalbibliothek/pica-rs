#[derive(Debug, PartialEq)]
pub enum ParsePicaError {
    InvalidSubfield,
    InvalidField,
    InvalidRecord,
}
