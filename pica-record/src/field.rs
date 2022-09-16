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
    pub fn tag(&self) -> &TagRef {
        &self.tag
    }

    pub fn occurrence(&self) -> Option<&OccurrenceRef> {
        self.occurrence.as_ref()
    }

    pub fn subfields(&self) -> &Vec<SubfieldRef> {
        &self.subfields
    }
}
