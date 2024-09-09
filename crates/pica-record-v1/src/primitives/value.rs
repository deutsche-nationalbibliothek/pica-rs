use std::fmt::{self, Display};
use std::ops::Deref;

use bstr::{BStr, ByteSlice};

use crate::PicaError;

/// An immutable PICA+ subfield value.
///
/// This type behaves like byte slice but guarantees that the subfield
/// value does contain neither '\x1e' nor '\x1f'.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct SubfieldValueRef<'a>(&'a BStr);

impl<'a> SubfieldValueRef<'a> {
    /// Create a new subfield value reference from a byte slice.
    ///
    /// # Error
    ///
    /// This function fails if the subfield value contains either the
    /// field separator '\x1f' or the record separator '\x1e'.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::SubfieldValueRef;
    ///
    /// let value = SubfieldValueRef::new(b"abc")?;
    /// assert_eq!(value, "abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new<T>(value: &'a T) -> Result<Self, PicaError>
    where
        T: AsRef<[u8]> + ?Sized,
    {
        let value = value.as_ref();
        if value.find_byteset(b"\x1f\x1e").is_some() {
            return Err(PicaError::InvalidSubfieldValue(
                value.to_str_lossy().to_string(),
            ));
        }

        Ok(Self(value.into()))
    }

    /// Create a new subfield value reference from a byte slice without
    /// checking for validity.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that the value neither contains the
    /// record separator '\x1e' nor the field separator '\x1f'.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::SubfieldValueRef;
    ///
    /// let value = SubfieldValueRef::from_unchecked("abc");
    /// assert_eq!(value, "abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_unchecked<T>(value: &'a T) -> Self
    where
        T: AsRef<[u8]> + ?Sized,
    {
        Self(value.as_ref().into())
    }

    /// Returns the subfield value as a byte slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::SubfieldValueRef;
    ///
    /// let value = SubfieldValueRef::from_unchecked("abc");
    /// assert_eq!(value.as_bytes(), b"abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn as_bytes(&self) -> &'a [u8] {
        self.0
    }
}

impl<'a> Deref for SubfieldValueRef<'a> {
    type Target = BStr;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Display for SubfieldValueRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq<str> for SubfieldValueRef<'_> {
    fn eq(&self, value: &str) -> bool {
        self.0 == value.as_bytes()
    }
}

impl PartialEq<&str> for SubfieldValueRef<'_> {
    fn eq(&self, value: &&str) -> bool {
        self.0 == value.as_bytes()
    }
}

impl PartialEq<Vec<u8>> for SubfieldValueRef<'_> {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.0 == other
    }
}

/// A mutable PICA+ subfield value.
///
/// This type behaves like byte slice but guarantees that the subfield
/// value does not contain neither '\x1e' or '\x1f'.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct SubfieldValue(Vec<u8>);

impl SubfieldValue {
    /// Create a new subfield value from a byte slice.
    ///
    /// # Error
    ///
    /// This function fails if the subfield value contains either the
    /// field separator '\x1f' or the record separator '\x1e'.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::SubfieldValue;
    ///
    /// let value = SubfieldValue::new(b"abc")?;
    /// assert_eq!(value, "abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new<T>(value: &T) -> Result<Self, PicaError>
    where
        T: AsRef<[u8]>,
    {
        let value = value.as_ref();
        if value.find_byteset(b"\x1f\x1e").is_some() {
            return Err(PicaError::InvalidSubfieldValue(
                value.to_str_lossy().to_string(),
            ));
        }

        Ok(Self(value.to_vec()))
    }

    /// Create a new subfield value from a byte slice without checking
    /// for validity.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that the value neither contains the
    /// record separator '\x1e' nor the field separator '\x1f'.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::SubfieldValue;
    ///
    /// let value = SubfieldValue::from_unchecked("abc");
    /// assert_eq!(value, "abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_unchecked<T>(value: &T) -> Self
    where
        T: AsRef<[u8]> + ?Sized,
    {
        Self(value.as_ref().to_vec())
    }
}

impl Display for SubfieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.as_bstr())
    }
}

impl From<SubfieldValueRef<'_>> for SubfieldValue {
    fn from(value: SubfieldValueRef<'_>) -> Self {
        Self(value.to_vec())
    }
}

impl PartialEq<SubfieldValueRef<'_>> for SubfieldValue {
    fn eq(&self, other: &SubfieldValueRef<'_>) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<SubfieldValue> for SubfieldValueRef<'_> {
    fn eq(&self, other: &SubfieldValue) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<&str> for SubfieldValue {
    fn eq(&self, other: &&str) -> bool {
        self.0 == other.as_bytes()
    }
}

#[cfg(feature = "arbitrary")]
impl quickcheck::Arbitrary for SubfieldValue {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let value = String::arbitrary(g).replace(['\x1f', '\x1e'], "");
        Self::from_unchecked(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subfield_value_ref_new() {
        let value = SubfieldValueRef::new("abc").unwrap();
        assert_eq!(value, "abc");

        let value = SubfieldValueRef::new("").unwrap();
        assert_eq!(value, "");

        assert_eq!(
            SubfieldValueRef::new("abc\x1e").unwrap_err(),
            PicaError::InvalidSubfieldValue("abc\x1e".to_string())
        );

        assert_eq!(
            SubfieldValueRef::new("abc\x1f").unwrap_err(),
            PicaError::InvalidSubfieldValue("abc\x1f".to_string())
        );
    }

    #[test]
    fn test_subfield_value_ref_from_unchecked() {
        let value = SubfieldValueRef::from_unchecked("abc");
        assert_eq!(value, "abc");

        let value = SubfieldValueRef::from_unchecked("");
        assert_eq!(value, "");
    }

    #[test]
    fn test_subfield_value_ref_as_bytes() {
        let value = SubfieldValueRef::from_unchecked("abc");
        assert_eq!(value.as_bytes(), b"abc");
    }
}
