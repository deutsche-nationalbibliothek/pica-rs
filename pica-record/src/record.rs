use std::fmt::Display;
use std::slice::Iter;

use bstr::{BStr, BString};

use crate::Field;

/// A PICA+ record.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Record<T>(pub(crate) Vec<Field<T>>);

/// A immutable PICA+ record.
pub type RecordRef<'a> = Record<&'a BStr>;

/// A mutable PICA+ tag.
pub type RecordMut = Record<BString>;

impl<'a, T: AsRef<[u8]> + From<&'a BStr> + Display> Record<T> {
    /// Create a new record.
    ///
    /// # Panics
    ///
    /// This method panics if a parameter is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::RecordRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let record =
    ///         RecordRef::new(vec![("003@", None, vec![('0', "abc")])]);
    ///     assert_eq!(record.iter().len(), 1);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<U: Into<T>>(
        fields: Vec<(U, Option<U>, Vec<(char, U)>)>,
    ) -> Self {
        let fields = fields
            .into_iter()
            .map(|(tag, occurrence, subfields)| {
                Field::new(tag, occurrence, subfields)
            })
            .collect();

        Self(fields)
    }

    /// Returns an iterator over the fields of the record.
    ///
    /// # Panics
    ///
    /// This method panics if a parameter is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::RecordRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let record = RecordRef::new(vec![
    ///         ("003@", None, vec![('0', "123456789X")]),
    ///         ("002@", None, vec![('0', "Oaf")]),
    ///     ]);
    ///
    ///     assert_eq!(record.iter().len(), 2);
    ///     Ok(())
    /// }
    /// ```
    pub fn iter(&self) -> Iter<Field<T>> {
        self.0.iter()
    }

    /// Returns `true` if the record contains no fields, otherwise
    /// `false`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::RecordRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let record =
    ///         RecordRef::new(vec![("002@", None, vec![('0', "Oaf")])]);
    ///     assert!(!record.is_empty());
    ///     Ok(())
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }
}
