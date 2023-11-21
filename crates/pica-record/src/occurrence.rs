use std::fmt::{self, Display};
use std::io::{self, Write};

use bstr::{BStr, BString, ByteSlice};
use winnow::combinator::preceded;
use winnow::stream::AsChar;
use winnow::token::take_while;
use winnow::{PResult, Parser};

use crate::ParsePicaError;

/// An immutable PICA+ occurrence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct OccurrenceRef<'a>(&'a BStr);

/// A mutable PICA+ occurrence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Occurrence(BString);

impl<'a> OccurrenceRef<'a> {
    /// Create an immutable PICA+ occurrence.
    ///
    /// # Panics
    ///
    /// This method panics if the occurrence is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::OccurrenceRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let occurrence = OccurrenceRef::new("01");
    ///     assert_eq!(occurrence, "01");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<B: ?Sized + AsRef<[u8]>>(value: &'a B) -> Self {
        Self::try_from(value.as_ref()).expect("value occurrence")
    }

    /// Creates an immutable PICA+ tag from a byte slice.
    ///
    /// If an invalid tag is given, an error is returned.
    ///
    /// ```rust
    /// use pica_record::OccurrenceRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     assert!(OccurrenceRef::from_bytes(b"/01").is_ok());
    ///     assert!(OccurrenceRef::from_bytes(b"01").is_err());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ParsePicaError> {
        parse_occurrence
            .parse(bytes)
            .map_err(|_| ParsePicaError::InvalidOccurrence)
    }

    /// Converts a occurrence reference into the underlying byte slice.
    ///
    /// ```rust
    /// use pica_record::OccurrenceRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let occurrence = OccurrenceRef::from_bytes(b"/01")?;
    ///     assert_eq!(occurrence.as_bytes(), b"01");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    /// Creates an immutable PICA+ tag from a unchecked byte string.
    ///
    /// ```rust
    /// use pica_record::OccurrenceRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     assert_eq!(OccurrenceRef::new(b"01"), "01");
    ///     assert_ne!(OccurrenceRef::new(b"01"), "02");
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn from_unchecked(value: &'a BStr) -> Self {
        Self(value)
    }

    /// Write the occurrence into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::OccurrenceRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let mut writer = Cursor::new(Vec::<u8>::new());
    ///     let occurrence = OccurrenceRef::new("01");
    ///     occurrence.write_to(&mut writer);
    ///     #
    ///     # assert_eq!(
    ///     #    String::from_utf8(writer.into_inner())?,
    ///     #    "/01"
    ///     # );
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        write!(out, "/{}", self.0)
    }
}

impl<'a, T: AsRef<[u8]>> PartialEq<T> for OccurrenceRef<'a> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

impl<'a> Display for OccurrenceRef<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}", self.0)
    }
}

/// Parse the digits of an PICA+ occurrence.
#[inline]
pub fn parse_occurrence_digits<'a>(
    i: &mut &'a [u8],
) -> PResult<&'a BStr> {
    take_while(2..=3, AsChar::is_dec_digit)
        .map(ByteSlice::as_bstr)
        .parse_next(i)
}

/// Parse a PICA+ occurrence (read-only).
#[inline]
pub(crate) fn parse_occurrence<'a>(
    i: &mut &'a [u8],
) -> PResult<OccurrenceRef<'a>> {
    preceded(b'/', parse_occurrence_digits)
        .map(|value| OccurrenceRef(value.as_bstr()))
        .parse_next(i)
}

impl<'a> TryFrom<&'a [u8]> for OccurrenceRef<'a> {
    type Error = ParsePicaError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if parse_occurrence_digits.parse(value).is_err() {
            return Err(ParsePicaError::InvalidOccurrence);
        }

        Ok(Self(value.into()))
    }
}

impl Occurrence {
    /// Converts a occurrence into the underlying byte slice.
    ///
    /// ```rust
    /// use pica_record::{Occurrence, OccurrenceRef};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let occurrence = Occurrence::from(OccurrenceRef::new("01"));
    ///     assert_eq!(occurrence.as_bytes(), b"01");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    /// Write the occurrence into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::{Occurrence, OccurrenceRef};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let mut writer = Cursor::new(Vec::<u8>::new());
    ///     let occurrence: Occurrence = OccurrenceRef::new("01").into();
    ///     occurrence.write_to(&mut writer);
    ///     #
    ///     # assert_eq!(
    ///     #    String::from_utf8(writer.into_inner())?,
    ///     #    "/01"
    ///     # );
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        write!(out, "/{}", self.0)
    }
}

impl From<OccurrenceRef<'_>> for Occurrence {
    fn from(value: OccurrenceRef<'_>) -> Self {
        Self(value.0.into())
    }
}

impl AsRef<[u8]> for Occurrence {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg(feature = "arbitrary")]
impl quickcheck::Arbitrary for Occurrence {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let size = *g.choose(&[2, 3]).unwrap();
        let value = (0..size)
            .map(|_| *g.choose(b"0123456789").unwrap())
            .collect::<Vec<u8>>();

        Occurrence(value.into())
    }
}

#[cfg(test)]
mod tests {
    use bstr::ByteSlice;

    use super::*;

    #[quickcheck_macros::quickcheck]
    fn parse_arbitrary_occurrence(occurrence: Occurrence) -> bool {
        let mut bytes = Vec::<u8>::new();
        let _ = occurrence.write_to(&mut bytes);
        super::parse_occurrence.parse(&bytes).is_ok()
    }

    #[test]
    fn parse_occurrence_digits() {
        use super::parse_occurrence_digits;

        macro_rules! parse_success {
            ($input:expr) => {
                assert_eq!(
                    parse_occurrence_digits.parse($input).unwrap(),
                    $input.as_bstr()
                );
            };
        }

        parse_success!(b"00");
        parse_success!(b"01");
        parse_success!(b"000");
        parse_success!(b"001");

        assert!(parse_occurrence_digits.parse(b"").is_err());
        assert!(parse_occurrence_digits.parse(b"0").is_err());
        assert!(parse_occurrence_digits.parse(b"0001").is_err());
        assert!(parse_occurrence_digits.parse(b"0a").is_err());
    }

    #[test]
    fn parse_occurrence() {
        macro_rules! parse_success {
            ($input:expr) => {
                assert_eq!(
                    super::parse_occurrence.parse($input).unwrap(),
                    OccurrenceRef($input[1..].as_bstr())
                );
            };
        }

        parse_success!(b"/00");
        parse_success!(b"/000");
        parse_success!(b"/001");
        parse_success!(b"/01");

        macro_rules! parse_error {
            ($input:expr) => {
                assert!(super::parse_occurrence.parse($input).is_err());
            };
        }

        parse_error!(b"");
        parse_error!(b"/");
        parse_error!(b"/0a");
        parse_error!(b"/0001");
        parse_error!(b"/0");
    }
}
