use bstr::BStr;

use crate::{OccurrenceRef, SubfieldRef, TagRef};

#[derive(Debug)]
pub struct FieldRef<'a> {
    tag: TagRef<'a>,
    occurrence: Option<OccurrenceRef<'a>>,
    subfields: Vec<SubfieldRef<'a>>,
}

impl<'a> FieldRef<'a> {
    /// Create a new field reference.
    ///
    /// # Panics
    ///
    /// This method panics if a parameter is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field =
    ///         FieldRef::new("003@", None, vec![('0', "123456789X")]);
    ///     assert_eq!(field.tag(), "003@");
    ///     assert!(field.occurrence().is_none());
    ///     assert_eq!(field.subfields().len(), 1);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: Into<&'a BStr>>(
        tag: T,
        occurrence: Option<T>,
        subfields: Vec<(char, T)>,
    ) -> FieldRef<'a> {
        let occurrence =
            occurrence.map(|digits| OccurrenceRef::new(digits.into()));
        let subfields = subfields
            .into_iter()
            .map(|(code, value)| SubfieldRef::new(code, value))
            .collect();

        FieldRef {
            tag: TagRef::new(tag),
            occurrence,
            subfields,
        }
    }

    /// Returns the tag of the field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field =
    ///         FieldRef::new("003@", None, vec![('0', "123456789X")]);
    ///     assert_eq!(field.tag(), "003@");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn tag(&self) -> &TagRef {
        &self.tag
    }

    /// Returns a reference to the occurrence of the field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::{FieldRef, OccurrenceRef};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field = FieldRef::new("012A", Some("01"), vec![]);
    ///     let occurrence = field.occurrence().unwrap();
    ///     assert_eq!(occurrence, "01");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn occurrence(&self) -> Option<&OccurrenceRef> {
        self.occurrence.as_ref()
    }

    /// Returns the subfields of the field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field = FieldRef::new(
    ///         "012A",
    ///         Some("01"),
    ///         vec![('a', "b"), ('c', "d")],
    ///     );
    ///
    ///     assert_eq!(field.subfields().len(), 2);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn subfields(&self) -> &Vec<SubfieldRef> {
        self.subfields.as_ref()
    }
}
