use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq)]
pub struct Subfield<'a> {
    pub code: char,
    pub value: Cow<'a, str>,
}

impl<'a> Subfield<'a> {
    /// Create a new subfield
    ///
    /// # Example
    /// ```
    /// use pica::Subfield;
    ///
    /// let subfield = Subfield::new('a', "foo");
    /// assert_eq!(subfield.code, 'a');
    /// ```
    pub fn new<S>(code: char, value: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self {
            code,
            value: value.into(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Field<'a, 'b, 'c> {
    pub tag: Cow<'a, str>,
    pub occurrence: Cow<'b, str>,
    pub subfields: Vec<Subfield<'c>>,
}

impl<'a, 'b, 'c> Field<'a, 'b, 'c> {
    pub fn new<S, T>(
        tag: S,
        occurrence: T,
        subfields: Vec<Subfield<'c>>,
    ) -> Self
    where
        S: Into<Cow<'a, str>>,
        T: Into<Cow<'b, str>>,
    {
        Self {
            tag: tag.into(),
            occurrence: occurrence.into(),
            subfields,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Record<'a, 'b, 'c> {
    pub fields: Vec<Field<'a, 'b, 'c>>,
}
