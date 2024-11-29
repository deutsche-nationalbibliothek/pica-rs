use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, stdin, BufRead, BufReader, Read};
use std::path::Path;
use std::str::Utf8Error;

use flate2::read::GzDecoder;

use crate::primitives::ParsePicaError;
use crate::{ByteRecord, StringRecord};

/// An error that can occur when reading records.
#[derive(thiserror::Error, Debug)]
pub enum ReadPicaError {
    #[error("parse erorr: {msg}")]
    Parse { msg: String, err: ParsePicaError },
    #[error("parse erorr: {msg}")]
    Utf8 { msg: String, err: Utf8Error },
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

impl ReadPicaError {
    /// Returns true if the error variant is a parse error and the
    /// `skip_invalid` flag is true.
    #[inline(always)]
    pub fn skip_parse_err(&self, skip_invalid: bool) -> bool {
        matches!(self, Self::Parse { .. }) && skip_invalid
    }
}

/// Configures and builda a PICA+ reader.
#[derive(Debug, Default)]
pub struct ReaderBuilder();

impl ReaderBuilder {
    /// Creates a new [ReaderBuilder].
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::prelude::*;
    ///
    /// let data = Cursor::new(b"002@ \x1f0Abvz\x1e\n");
    /// let _reader = ReaderBuilder::new().from_reader(data, None);
    /// let _reader =
    ///     ReaderBuilder::new().from_path("tests/data/ada.dat")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [ReaderBuilder] from an existing reader.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::prelude::*;
    ///
    /// let data = Cursor::new(b"002@ \x1f0Abvz\x1e\n");
    /// let reader = ReaderBuilder::new().from_reader(data, None);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_reader<R: Read>(
        &self,
        reader: R,
        source: Option<String>,
    ) -> Reader<R> {
        Reader::new(self, reader, source)
    }

    /// Creates a new [ReaderBuilder] from a path.
    ///
    /// # Note
    ///
    /// A path equal to "-" means reading from stdin.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::prelude::*;
    ///
    /// let reader = ReaderBuilder::new().from_path("-")?;
    /// let reader =
    ///     ReaderBuilder::new().from_path("tests/data/DUMP.dat.gz")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_path<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> io::Result<Reader<Box<dyn Read>>> {
        let path = path.as_ref();
        let source = path.to_str().map(ToString::to_string);
        let reader: Box<dyn Read> = match path
            .extension()
            .and_then(OsStr::to_str)
        {
            Some("gz") => Box::new(GzDecoder::new(File::open(path)?)),
            _ => {
                if path.to_str() != Some("-") {
                    Box::new(File::open(path)?)
                } else {
                    Box::new(stdin().lock())
                }
            }
        };

        Ok(self.from_reader(reader, source))
    }
}

pub struct Reader<R: Read> {
    inner: BufReader<R>,
    source: Option<String>,
    line: usize,
    buf: Vec<u8>,
}

impl<R: Read> Reader<R> {
    /// Creates a new [Reader].
    pub(crate) fn new(
        _builder: &ReaderBuilder,
        reader: R,
        source: Option<String>,
    ) -> Self {
        let source = source.map(|s| {
            if s == "-" {
                "<stdin>".to_string()
            } else {
                s
            }
        });

        Self {
            inner: BufReader::new(reader),
            buf: Vec::new(),
            line: 0,
            source,
        }
    }

    /// Consumes the reader and returns the underlying [BufReader].
    pub fn into_inner(self) -> BufReader<R> {
        self.inner
    }
}

pub trait RecordsIter {
    type ByteItem<'a>
    where
        Self: 'a;

    type StringItem<'a>
    where
        Self: 'a;

    fn next_byte_record(&mut self) -> Option<Self::ByteItem<'_>>;
    fn next_string_record(&mut self) -> Option<Self::StringItem<'_>>;
}

impl<R: Read> RecordsIter for Reader<R> {
    type ByteItem<'a>
        = Result<ByteRecord<'a>, ReadPicaError>
    where
        Self: 'a;

    type StringItem<'a>
        = Result<StringRecord<'a>, ReadPicaError>
    where
        Self: 'a;

    /// Advance the iterator and return the next [ByteRecord].
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::prelude::*;
    ///
    /// let data = Cursor::new(b"002@ \x1f0Abvz\x1e\n");
    /// let mut reader = ReaderBuilder::new().from_reader(data, None);
    ///
    /// let mut count = 0;
    /// while let Some(result) = reader.next_byte_record() {
    ///     if result.is_ok() {
    ///         count += 1
    ///     }
    /// }
    ///
    /// assert_eq!(count, 1);
    ///
    /// # // a byte record can contain invalid unicode data.
    /// # let data = Cursor::new(b"002@ \x1f0Abvz\xff\xfd\x1e\n");
    /// # let mut reader = ReaderBuilder::new().from_reader(data, None);
    /// # let record = reader.next_byte_record().unwrap();
    /// # assert!(record.is_ok());
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn next_byte_record(&mut self) -> Option<Self::ByteItem<'_>> {
        self.buf.clear();
        self.line += 1;

        match self.inner.read_until(b'\n', &mut self.buf) {
            Err(e) => Some(Err(ReadPicaError::from(e))),
            Ok(0) => None,
            Ok(_) => match ByteRecord::from_bytes(&self.buf) {
                Ok(record) => Some(Ok(record)),
                Err(err) => {
                    let msg = if let Some(ref src) = self.source {
                        format!(
                            "invalid record on line {} ({src}).",
                            self.line
                        )
                    } else {
                        format!("invalid record on line {}.", self.line)
                    };

                    Some(Err(ReadPicaError::Parse { msg, err }))
                }
            },
        }
    }

    /// Advance the iterator and return the next [StringRecord].
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::prelude::*;
    ///
    /// let data = Cursor::new(b"002@ \x1f0Abvz\x1e\n");
    /// let mut reader = ReaderBuilder::new().from_reader(data, None);
    ///
    /// let mut count = 0;
    /// while let Some(result) = reader.next_string_record() {
    ///     if result.is_ok() {
    ///         count += 1
    ///     }
    /// }
    ///
    /// assert_eq!(count, 1);
    ///
    /// # // invalid unicode data results in an error.
    /// # let data = Cursor::new(b"002@ \x1f0Abvz\xff\xfd\x1e\n");
    /// # let mut reader = ReaderBuilder::new().from_reader(data, None);
    /// # let record = reader.next_string_record().unwrap();
    /// # assert!(record.is_err());
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn next_string_record(&mut self) -> Option<Self::StringItem<'_>> {
        let line = self.line;
        match self.next_byte_record() {
            Some(Ok(record)) => match StringRecord::try_from(record) {
                Ok(record) => Some(Ok(record)),
                Err(err) => {
                    let msg = format!("invalid record on line {line}.");
                    Some(Err(ReadPicaError::Utf8 { msg, err }))
                }
            },
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }
}
