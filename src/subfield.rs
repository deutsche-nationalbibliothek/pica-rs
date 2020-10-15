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
