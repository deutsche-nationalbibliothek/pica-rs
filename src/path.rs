//! Pica+ Path
//!
//! A path is a query syntax to address values within a pica+ record. The path
//! consists of a [`crate::Field`] tag and a [`crate::Subfield`] name. A
//! [`crate::Field`] occurrence or an index is optional
//!
//! # Grammar
//!
//! ```text
//! path       ::= tag occurrence? name
//! tag        ::= [012] [0-9]{2} ([A-Z] | '@')
//! occurrence ::= '/' [0-9]{2,3}
//! name       ::= [a-z] | [A-Z] | [0-9]
//! ```
use pica_core::Tag;
use pica_matcher::TagMatcher;

use crate::matcher::OccurrenceMatcher;
use crate::parser::{parse_path, ParsePathError};
use crate::{Error, Result};

use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct Path {
    pub(crate) tag: TagMatcher,
    pub(crate) occurrence: OccurrenceMatcher,
    pub(crate) codes: Vec<char>,
}

impl Path {
    /// Creates a new path
    ///
    /// ```rust
    /// use pica::matcher::OccurrenceMatcher;
    /// use pica::Path;
    ///
    /// assert!(Path::new("003@", OccurrenceMatcher::None, vec!['0']).is_ok());
    /// assert!(Path::new("012A", OccurrenceMatcher::Any, vec!['0']).is_ok());
    /// assert!(Path::new("012!", OccurrenceMatcher::Any, vec!['0']).is_err());
    /// assert!(Path::new("012A", OccurrenceMatcher::Any, vec!['a', '!']).is_err());
    /// ```
    pub fn new<S>(
        tag: S,
        occurrence: OccurrenceMatcher,
        codes: Vec<char>,
    ) -> Result<Path>
    where
        S: AsRef<str>,
    {
        for code in &codes {
            if !code.is_ascii_alphanumeric() {
                return Err(Error::InvalidSubfield(format!(
                    "Invalid subfield code '{}' in path expression.",
                    code
                )));
            }
        }

        Ok(Path {
            tag: TagMatcher::Some(Tag::from_bytes(tag.as_ref().as_bytes())?),
            occurrence,
            codes,
        })
    }

    /// Creates a new `Path` from a byte vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::matcher::OccurrenceMatcher;
    /// use pica::Path;
    /// use pica_core::Occurrence;
    /// use std::str::FromStr;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let path = Path::from_bytes("003@.0")?;
    ///     assert_eq!(
    ///         path,
    ///         Path::new("003@", OccurrenceMatcher::None, vec!['0'])?
    ///     );
    ///
    ///     let path = Path::from_bytes("012A/01.0")?;
    ///     assert_eq!(
    ///         path,
    ///         Path::new(
    ///             "012A",
    ///             OccurrenceMatcher::Some(Occurrence::from_str("/01")?),
    ///             vec!['0']
    ///         )?
    ///     );
    ///
    ///     let path = Path::from_bytes("012A/*.0")?;
    ///     assert_eq!(path, Path::new("012A", OccurrenceMatcher::Any, vec!['0'])?);
    ///
    ///     let path = Path::from_bytes("012A/*.[abc]")?;
    ///     assert_eq!(
    ///         path,
    ///         Path::new("012A", OccurrenceMatcher::Any, vec!['a', 'b', 'c'])?
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes<T>(data: T) -> std::result::Result<Path, ParsePathError>
    where
        T: Into<Vec<u8>>,
    {
        match parse_path(&data.into()) {
            Err(_) => {
                Err(ParsePathError(String::from("Invalid path expression")))
            }
            Ok((_, path)) => Ok(path),
        }
    }
}

impl FromStr for Path {
    type Err = crate::error::Error;

    /// Parse a `Path` from a string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Path;
    /// use std::str::FromStr;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let path = Path::from_str("003@.0");
    ///     assert!(path.is_ok());
    ///
    ///     Ok(())
    /// }
    /// ```
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::from_bytes(s)?)
    }
}
