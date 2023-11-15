use std::io::{self, Write};
use std::iter;
use std::str::Utf8Error;

use bstr::{BStr, ByteSlice};
use winnow::combinator::preceded;
use winnow::token::{one_of, take_till0};
use winnow::{PResult, Parser};

use crate::error::ParsePicaError;

/// An immutable PICA+ subfield.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subfield<'a> {
    code: char,
    value: &'a BStr,
}

/// Parse a PICA+ subfield code.
#[inline]
fn parse_subfield_code(i: &mut &[u8]) -> PResult<char> {
    one_of((b'0'..=b'9', b'a'..=b'z', b'A'..=b'Z'))
        .map(char::from)
        .parse_next(i)
}

/// Parse a PICA+ subfield value.
#[inline]
fn parse_subfield_value<'a>(i: &mut &'a [u8]) -> PResult<&'a BStr> {
    take_till0(|c| c == b'\x1f' || c == b'\x1e')
        .map(ByteSlice::as_bstr)
        .parse_next(i)
}

/// Parse a PICA+ subfield.
#[inline]
pub(crate) fn parse_subfield<'a>(
    i: &mut &'a [u8],
) -> PResult<Subfield<'a>> {
    preceded(b'\x1f', (parse_subfield_code, parse_subfield_value))
        .map(|(code, value)| Subfield { code, value })
        .parse_next(i)
}

impl<'a> Subfield<'a> {
    /// Create a new subfield.
    ///
    /// # Panics
    ///
    /// This method panics if the subfield code or the value is
    /// invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = Subfield::new('a', "bcd");
    ///     assert_eq!(subfield.code(), 'a');
    ///     assert_eq!(subfield.value(), "bcd");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T>(code: char, value: &'a T) -> Self
    where
        T: ?Sized + AsRef<[u8]>,
    {
        Self::try_from((code, value)).expect("valid subfield")
    }

    /// Creates an immutable PICA+ subfield from a byte slice.
    ///
    /// If an invalid subfield is given, an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = Subfield::from_bytes(b"\x1f0123456789X")?;
    ///
    ///     assert_eq!(subfield.code(), '0');
    ///     assert_eq!(subfield.value(), "123456789X");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ParsePicaError> {
        parse_subfield
            .parse(bytes)
            .map_err(|_| ParsePicaError::InvalidSubfield)
    }

    /// Returns the code of the subfield.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = Subfield::new('0', "0123456789X");
    ///     assert_eq!(subfield.code(), '0');
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn code(&self) -> char {
        self.code
    }

    /// Returns the value of the subfield.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = Subfield::new('0', "0123456789X");
    ///     assert_eq!(subfield.value(), "0123456789X");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn value(&self) -> &BStr {
        self.value
    }

    /// Returns true if the subfield value is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = Subfield::new('0', "abc");
    ///     assert!(!subfield.is_empty());
    ///
    ///     let subfield = Subfield::new('0', "");
    ///     assert!(subfield.is_empty());
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Returns an [`std::str::Utf8Error`](Utf8Error) if the subfield
    /// value contains invalid UTF-8 data, otherwise the unit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = Subfield::new('0', "123456789X");
    ///     assert!(subfield.validate().is_ok());
    ///
    ///     let subfield = Subfield::from_bytes(&[b'\x1f', b'0', 0, 159])?;
    ///     assert_eq!(subfield.validate().is_err(), true);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn validate(&self) -> Result<(), Utf8Error> {
        if self.value.is_ascii() {
            return Ok(());
        }

        std::str::from_utf8(self.value)?;
        Ok(())
    }

    /// Write the subfield into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let mut writer = Cursor::new(Vec::<u8>::new());
    ///     let subfield = Subfield::new('0', "123456789X");
    ///     subfield.write_to(&mut writer);
    ///     #
    ///     # assert_eq!(
    ///     #    String::from_utf8(writer.into_inner())?,
    ///     #    "\x1f0123456789X"
    ///     # );
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        write!(out, "\x1f{}{}", self.code, self.value)
    }
}

impl<'a> IntoIterator for &'a Subfield<'a> {
    type Item = &'a Subfield<'a>;
    type IntoIter = iter::Once<Self::Item>;

    /// Creates an iterator from a single subfield. The iterator just
    /// returns the subfield once.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = Subfield::new('0', "123456789X");
    ///     let mut iter = subfield.into_iter();
    ///
    ///     assert_eq!(iter.next(), Some(&subfield));
    ///     assert_eq!(iter.next(), None);
    ///
    ///     Ok(())
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        iter::once(self)
    }
}

impl<'a, T> TryFrom<(char, &'a T)> for Subfield<'a>
where
    T: ?Sized + AsRef<[u8]>,
{
    type Error = ParsePicaError;

    fn try_from(value: (char, &'a T)) -> Result<Self, Self::Error> {
        let (code, value) = (value.0, value.1.as_ref().as_bstr());

        if value.find_byteset(b"\x1e\x1f").is_some()
            || !code.is_ascii_alphanumeric()
        {
            return Err(ParsePicaError::InvalidSubfield);
        }

        Ok(Self { code, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bstr::ByteSlice;
    use std::io::Cursor;

    #[test]
    fn parse_subfield_code() {
        use super::parse_subfield_code;

        for c in b'0'..=b'z' {
            if c.is_ascii_alphanumeric() {
                assert_eq!(
                    parse_subfield_code.parse(&[c]).unwrap(),
                    c as char
                );
            } else {
                assert!(parse_subfield_code.parse(&[c]).is_err());
            }
        }
    }

    #[test]
    fn parse_subfield_value() {
        use super::parse_subfield_value;

        macro_rules! parse_success {
            ($input:expr, $expected:expr, $rest:expr) => {
                assert_eq!(
                    parse_subfield_value.parse_peek($input).unwrap(),
                    ($rest.as_bytes(), $expected.as_bstr())
                );
            };
        }

        parse_success!(b"abc", b"abc", b"");
        parse_success!(b"a\x1ebc", b"a", b"\x1ebc");
        parse_success!(b"a\x1fbc", b"a", b"\x1fbc");
        parse_success!(b"", b"", b"");
    }

    #[test]
    fn parse_subfield() {
        use super::parse_subfield;

        assert_eq!(
            parse_subfield.parse(b"\x1fa123").unwrap(),
            Subfield::new('a', "123")
        );

        assert_eq!(
            parse_subfield.parse(b"\x1fa").unwrap(),
            Subfield::new('a', "")
        );

        assert!(parse_subfield.parse(b"a123").is_err());
        assert!(parse_subfield.parse(b"").is_err());
    }

    #[test]
    fn subfield_new() {
        let _ = Subfield::new('a', "123");
        let _ = Subfield::new('a', "");
    }

    #[test]
    #[should_panic]
    fn subfield_new_panic_code() {
        Subfield::new('!', "123");
    }

    #[test]
    #[should_panic]
    fn subfield_new_panic_value() {
        Subfield::new('a', "a\x1eb");
    }

    #[test]
    fn from_bytes() {
        assert_eq!(
            Subfield::from_bytes(b"\x1fa123").unwrap(),
            Subfield::new('a', "123")
        );

        assert_eq!(
            Subfield::from_bytes(b"\x1f!123").unwrap_err(),
            ParsePicaError::InvalidSubfield
        );
    }

    #[test]
    fn try_from() {
        assert_eq!(
            Subfield::try_from(('a', "123")).unwrap(),
            Subfield::new('a', "123")
        );

        macro_rules! parse_error {
            ($input:expr) => {
                assert_eq!(
                    Subfield::try_from($input).unwrap_err(),
                    ParsePicaError::InvalidSubfield
                );
            };
        }

        parse_error!(('a', &[b'1', b'\x1e', b'2']));
        parse_error!(('a', &[b'1', b'\x1f', b'2']));
        parse_error!(('!', "123"));
    }

    #[test]
    fn subfield_code() {
        let subfield = Subfield::new('a', "123");
        assert_eq!(subfield.code(), 'a');
    }

    #[test]
    fn subfield_value() {
        let subfield = Subfield::new('a', "123");
        assert_eq!(subfield.value(), "123");
    }

    #[test]
    fn subfield_is_empty() {
        let subfield = Subfield::new('a', "123");
        assert!(!subfield.is_empty());

        let subfield = Subfield::new('a', "");
        assert!(subfield.is_empty());
    }

    #[test]
    fn subfield_validate() {
        let subfield = Subfield::new('a', "123");
        assert!(subfield.validate().is_ok());

        let subfield = Subfield::new('a', &[0, 159, 146, 150]);
        let error = subfield.validate().unwrap_err();
        assert_eq!(1, error.valid_up_to());
    }

    #[test]
    fn subfield_write_to() {
        let mut writer = Cursor::new(Vec::<u8>::new());
        let subfield = Subfield::new('a', "123");
        let _ = subfield.write_to(&mut writer);

        assert_eq!(
            writer.into_inner(),
            vec![b'\x1f', b'a', b'1', b'2', b'3']
        );
    }

    #[test]
    fn subfield_into_iter() {
        let subfield = Subfield::new('a', "123");
        let mut iter = subfield.into_iter();

        assert_eq!(iter.next(), Some(&subfield));
        assert_eq!(iter.next(), None);
    }
}
