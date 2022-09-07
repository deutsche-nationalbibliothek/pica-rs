use bstr::{BStr, ByteSlice};

/// An immutable PICA+ subfield.
#[derive(Debug, PartialEq, Eq)]
pub struct SubfieldRef<'a>(pub(crate) char, pub(crate) &'a BStr);

impl<'a> SubfieldRef<'a> {
    /// Create a new subfield reference.
    ///
    /// # Panics
    ///
    /// This method panics if the subfield code or the value is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = SubfieldRef::new('a', "bcd");
    ///     assert_eq!(subfield.code(), 'a');
    ///     assert_eq!(subfield.value(), "bcd");
    ///     Ok(())
    /// }
    /// ```
    pub fn new(code: char, value: impl Into<&'a BStr>) -> Self {
        let value = value.into();

        assert!(
            code.is_ascii_alphanumeric()
                && value.find_byte(b'\x1e').is_none()
                && value.find_byte(b'\x1f').is_none()
        );

        Self(code, value)
    }

    /// Returns the code of the subfield.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = SubfieldRef::new('0', "0123456789X");
    ///     assert_eq!(subfield.code(), '0');
    ///     Ok(())
    /// }
    /// ```
    pub fn code(&self) -> char {
        self.0
    }

    /// Returns the value of the subfield.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = SubfieldRef::new('0', "0123456789X");
    ///     assert_eq!(subfield.value(), "0123456789X");
    ///     Ok(())
    /// }
    /// ```
    pub fn value(&self) -> &'a BStr {
        self.1
    }
}
