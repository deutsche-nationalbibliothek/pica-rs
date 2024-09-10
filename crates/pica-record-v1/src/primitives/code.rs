use std::fmt::{self, Display};

use crate::PicaError;

/// A PICA+ subfield code.
///
/// This type represents a PICA+ subfield code, which is a ASCII
/// alpha-numeric chracter.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct SubfieldCode(char);

impl SubfieldCode {
    /// Creates a new subfield code.
    ///
    /// # Error
    ///
    /// This functions fails if the given code is not an ASCII
    /// alpha-numeric character.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::SubfieldCode;
    ///
    /// let code = SubfieldCode::new('a')?;
    /// assert_eq!(code, 'a');
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(code: char) -> Result<Self, PicaError> {
        if !code.is_ascii_alphanumeric() {
            return Err(PicaError::InvalidSubfieldCode(code));
        }

        Ok(Self(code))
    }

    /// Creates a subfied code without checking for validity.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that the given subfield code is valid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::SubfieldCode;
    ///
    /// let code = SubfieldCode::from_unchecked('a');
    /// assert_eq!(code, 'a');
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn from_unchecked<T: Into<char>>(code: T) -> Self {
        Self(code.into())
    }

    /// Returns the subfield code as a byte (`u8`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::SubfieldCode;
    ///
    /// let code = SubfieldCode::new('a')?;
    /// assert_eq!(code.as_byte(), b'a');
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn as_byte(&self) -> u8 {
        self.0 as u8
    }
}

impl Display for SubfieldCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq<char> for SubfieldCode {
    fn eq(&self, code: &char) -> bool {
        self.0 == *code
    }
}

impl PartialEq<char> for &SubfieldCode {
    fn eq(&self, code: &char) -> bool {
        self.0 == *code
    }
}

impl TryFrom<char> for SubfieldCode {
    type Error = PicaError;

    fn try_from(code: char) -> Result<Self, Self::Error> {
        Self::new(code)
    }
}

#[cfg(feature = "arbitrary")]
impl quickcheck::Arbitrary for SubfieldCode {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let code = (1..)
            .map(|_| char::arbitrary(g))
            .find(char::is_ascii_alphanumeric)
            .unwrap();

        Self(code)
    }
}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;

    use super::*;

    #[quickcheck]
    fn test_subfield_code_new(input: char) {
        let result = SubfieldCode::new(input);

        if input.is_ascii_alphanumeric() {
            assert_eq!(result.unwrap(), SubfieldCode(input));
        } else {
            assert_eq!(
                result.unwrap_err(),
                PicaError::InvalidSubfieldCode(input)
            );
        }
    }

    #[test]
    fn test_subfield_code_from_unchecked() {
        for c in '0'..='z' {
            if c.is_ascii_alphanumeric() {
                assert_eq!(
                    SubfieldCode::from_unchecked(c),
                    SubfieldCode(c)
                );
            }
        }
    }

    #[test]
    fn test_subfield_code_as_byte() {
        for c in '0'..='z' {
            if c.is_ascii_alphanumeric() {
                let code = SubfieldCode::new(c).unwrap();
                assert_eq!(code.as_byte(), c as u8);
            }
        }
    }

    #[test]
    fn test_subfield_code_try_from_char() {
        for c in '0'..='z' {
            if c.is_ascii_alphanumeric() {
                assert_eq!(
                    SubfieldCode::try_from(c).unwrap(),
                    SubfieldCode(c)
                );
            } else {
                assert_eq!(
                    SubfieldCode::try_from(c).unwrap_err(),
                    PicaError::InvalidSubfieldCode(c)
                );
            }
        }
    }
}
