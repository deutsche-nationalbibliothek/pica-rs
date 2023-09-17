use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

use flate2::read::GzDecoder;

use super::ReadPicaError;
use crate::ByteRecord;

/// Configures and builds a PICA+ reader.
#[derive(Debug, Default)]
pub struct ReaderBuilder {
    limit: usize,
}

impl ReaderBuilder {
    /// Create a new ReaderBuilder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Change the limit of records to read.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::{Cursor, Seek};
    ///
    /// use pica_record::io::{ReaderBuilder, RecordsIterator};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let data =
    ///         Cursor::new(b"003@ \x1f0abc\x1e\n003@ \x1f0def\x1e\n");
    ///     let mut reader =
    ///         ReaderBuilder::new().limit(1).from_reader(data, None);
    ///
    ///     let mut count = 0;
    ///     while let Some(result) = reader.next() {
    ///         count += 1;
    ///     }
    ///
    ///     assert_eq!(count, 1);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn limit(mut self, buffer_size: usize) -> Self {
        self.limit = buffer_size;
        self
    }

    /// ```rust
    /// use std::io::{Cursor, Seek};
    ///
    /// use pica_record::io::{ReaderBuilder, RecordsIterator};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let data =
    ///         Cursor::new(b"003@ \x1f0abc\x1e\n003@ \x1f0def\x1e\n");
    ///     let mut reader = ReaderBuilder::new().from_reader(data, None);
    ///
    ///     let mut count = 0;
    ///     while let Some(result) = reader.next() {
    ///         count += 1;
    ///     }
    ///
    ///     assert_eq!(count, 2);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_reader<R: Read>(
        &self,
        reader: R,
        source: Option<String>,
    ) -> Reader<R> {
        Reader::new(self, reader, source)
    }

    pub fn from_path<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> io::Result<Reader<Box<dyn Read>>> {
        let path = path.as_ref();
        let source = path.to_string_lossy().to_string();

        let reader: Box<dyn Read> = match path
            .extension()
            .and_then(OsStr::to_str)
        {
            Some("gz") => Box::new(GzDecoder::new(File::open(path)?)),
            _ => {
                if path.to_str() != Some("-") {
                    Box::new(File::open(path)?)
                } else {
                    Box::new(io::stdin())
                }
            }
        };

        Ok(self.from_reader(reader, Some(source)))
    }
}

pub struct Reader<R: Read> {
    inner: BufReader<R>,
    source: Option<String>,
    limit: usize,
    count: usize,
    buf: Vec<u8>,
}

impl<R: Read> Reader<R> {
    pub fn new(
        builder: &ReaderBuilder,
        reader: R,
        source: Option<String>,
    ) -> Self {
        Self {
            inner: BufReader::new(reader),
            limit: builder.limit,
            source,
            buf: vec![],
            count: 0,
        }
    }

    pub fn into_inner(self) -> BufReader<R> {
        self.inner
    }
}

pub trait RecordsIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>>;
}

impl<R: Read> RecordsIterator for Reader<R> {
    type Item<'a> = Result<ByteRecord<'a>, ReadPicaError> where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.limit > 0 && self.count >= self.limit {
            return None;
        }

        self.buf.clear();
        match self.inner.read_until(b'\n', &mut self.buf) {
            Err(e) => Some(Err(ReadPicaError::from(e))),
            Ok(0) => None,
            Ok(_) => {
                let result = ByteRecord::from_bytes(&self.buf);
                match result {
                    Err(err) => {
                        let msg = match &self.source {
                            Some(source) => {
                                if source == "-" {
                                    format!("invalid record in line {} (stdin)", self.count)
                                } else {
                                    format!("invalid record in line {} ({})", self.count, source)
                                }
                            }
                            None => format!(
                                "invalid record on line {}",
                                self.count
                            ),
                        };

                        Some(Err(ReadPicaError::Parse { msg, err }))
                    }
                    Ok(record) => {
                        self.count += 1;
                        Some(Ok(record))
                    }
                }
            }
        }
    }
}
