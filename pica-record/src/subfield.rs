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
    pub fn new(code: char, value: impl Into<&'a BStr>) -> Self {
        let value = value.into();

        assert!(
            code.is_ascii_alphanumeric()
                && value.find_byte(b'\x1e').is_none()
                && value.find_byte(b'\x1f').is_none()
        );

        Self(code, value)
    }
}
