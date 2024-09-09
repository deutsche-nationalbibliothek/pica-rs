use winnow::prelude::*;
use winnow::token::one_of;

use crate::error::ParsePicaError;

/// A PICA+ subfield code.
///
/// This type represents a PICA+ subfield code, which is a single ASCII
/// alpha-numeric character.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SubfieldCode(char);

impl SubfieldCode {
    /// Creates a new subfield code.
    ///
    /// # Errors
    ///
    /// This function fails if the given code is not an ASCII
    /// alpha-numeric character.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldCode;
    ///
    /// let code = SubfieldCode::new('a')?;
    /// assert_eq!(code, 'a');
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(code: char) -> Result<Self, ParsePicaError> {
        if !code.is_ascii_alphanumeric() {
            return Err(ParsePicaError::SubfieldCode(code));
        };

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
    /// use pica_record::SubfieldCode;
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
    /// use pica_record::SubfieldCode;
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

impl PartialEq<char> for SubfieldCode {
    /// Compares a [SubfieldCode] with a [char](std::char).
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldCode;
    ///
    /// let code = SubfieldCode::new('a')?;
    /// assert_eq!(code, 'a');
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, code: &char) -> bool {
        self.0 == *code
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for SubfieldCode {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        (1..)
            .map(|_| char::arbitrary(g))
            .find(char::is_ascii_alphanumeric)
            .map(SubfieldCode::from_unchecked)
            .unwrap()
    }
}

/// Parses a [SubfieldCode] from a byte slice.
#[allow(unused)]
pub(crate) fn parse_subfield_code(
    i: &mut &[u8],
) -> PResult<SubfieldCode> {
    one_of((b'0'..=b'9', b'a'..=b'z', b'A'..=b'Z'))
        .map(SubfieldCode::from_unchecked)
        .parse_next(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_subfield_code() {
        (u8::MIN..=u8::MAX).into_iter().for_each(|code| {
            if !code.is_ascii_alphanumeric() {
                assert!(parse_subfield_code.parse(&[code]).is_err());
            } else {
                assert_eq!(
                    parse_subfield_code.parse(&[code]).unwrap(),
                    SubfieldCode(code as char),
                )
            }
        });
    }
}
