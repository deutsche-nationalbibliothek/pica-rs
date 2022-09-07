use bstr::BStr;

/// An immutable PICA+ subfield.
#[derive(Debug, PartialEq, Eq)]
pub struct SubfieldRef<'a>(pub(crate) char, pub(crate) &'a BStr);
