use crate::parse;
use bstr::BStr;
use std::ops::Deref;

/// Represents a PICA+ subfield.
#[derive(Clone, Debug, PartialEq)]
pub struct Subfield<'a> {
    /// The name of the subfield, e.g. '0'.
    pub(crate) name: char,
    /// The value of the subfield, e.g. "Olfo".
    pub(crate) value: &'a BStr,
}

impl<'a> Subfield<'a> {
    /// Creates a new subfield.
    pub fn new<T>(name: char, value: T) -> Self
    where
        T: Into<&'a BStr>,
    {
        Self {
            name,
            value: value.into(),
        }
    }
}

/// Represents a PICA+ field.
#[derive(Debug, PartialEq)]
pub struct Field<'a> {
    /// The name of the field, e.g. "003@".
    pub(crate) name: &'a BStr,
    /// The (optional) occurrence of the field.
    pub(crate) occurrence: Option<&'a BStr>,
    /// The subfields of the field.
    pub(crate) subfields: Vec<Subfield<'a>>,
}

impl<'a> Field<'a> {
    /// Crates a new field.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the field.
    /// * `occurrence` - The occurrence of the field.
    /// * `subfields` - The subfields of the field.
    pub fn new<T>(
        name: T,
        occurrence: Option<T>,
        subfields: Vec<Subfield<'a>>,
    ) -> Self
    where
        T: Into<&'a BStr>,
    {
        Self {
            name: name.into(),
            occurrence: occurrence.map(|o| o.into()),
            subfields,
        }
    }
}

impl<'a> Deref for Field<'a> {
    type Target = Vec<Subfield<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.subfields
    }
}

/// Represents a PICA+ record.
#[derive(Debug, PartialEq)]
pub struct Record<'a>(pub(crate) Vec<Field<'a>>);

impl<'a> Record<'a> {
    /// Parses a record from a byte slice.
    #[allow(clippy::result_unit_err)]
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ()> {
        parse::record(data).map(|(_, s)| s).map_err(|_| ())
    }
}

impl<'a> Deref for Record<'a> {
    type Target = Vec<Field<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_from_bytes() {
        assert_eq!(
            Record::from_bytes(b"003@ \x1f0123456789X\x1e").unwrap(),
            Record(vec![Field::new(
                "003@",
                None,
                vec![Subfield::new('0', "123456789X")]
            )])
        )
    }
}
