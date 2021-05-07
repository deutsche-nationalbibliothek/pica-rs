use crate::error::{Error, Result};
use crate::parser::ParsePicaError;
use crate::{ByteRecord, StringRecord};
use flate2::read::GzDecoder;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::ops::{Deref, DerefMut};
use std::path::Path;

/// Configures and builds a PICA+ reader.
#[derive(Debug)]
pub struct ReaderBuilder {
    skip_invalid: bool,
    buffer_size: usize,
    limit: usize,
}

impl Default for ReaderBuilder {
    fn default() -> ReaderBuilder {
        ReaderBuilder {
            buffer_size: 65_536,
            skip_invalid: true,
            limit: 0,
        }
    }
}

impl ReaderBuilder {
    /// Create a new `ReaderBuilder` for reading PICA+ data.
    ///
    /// # Example
    ///
    /// ```
    /// use pica::{ByteRecord, ReaderBuilder, StringRecord};
    /// use std::error::Error;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let data = "003@ \x1f0123456789\x1e\n";
    ///     let mut reader = ReaderBuilder::new().from_reader(data.as_bytes());
    ///     let records = reader
    ///         .records()
    ///         .map(Result::unwrap)
    ///         .collect::<Vec<StringRecord>>();
    ///
    ///     assert_eq!(
    ///         records,
    ///         vec![StringRecord::from_byte_record(ByteRecord::from_bytes(
    ///             "003@ \x1f0123456789\x1e\n".as_bytes()
    ///         )?)?]
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn new() -> ReaderBuilder {
        ReaderBuilder::default()
    }

    /// Builds a new `Reader` with the current configuration, that reads from a
    /// file path.
    ///
    /// # Example
    ///
    /// ```
    /// use pica::ReaderBuilder;
    /// use std::error::Error;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let mut reader =
    ///         ReaderBuilder::new().from_path("tests/data/119232022.dat")?;
    ///     let record = reader.records().next().unwrap();
    ///     assert!(record.is_ok());
    ///
    ///     let mut reader =
    ///         ReaderBuilder::new().from_path("tests/data/119232022.dat.gz")?;
    ///     let record = reader.records().next().unwrap();
    ///     assert!(record.is_ok());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_path<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Reader<Box<dyn Read>>> {
        let filename = path.as_ref();

        let reader: Box<dyn Read> =
            if filename.extension() == Some(OsStr::new("gz")) {
                Box::new(GzDecoder::new(File::open(filename)?))
            } else {
                Box::new(File::open(filename)?)
            };

        Ok(self.from_reader(reader))
    }

    /// Builds a new `Reader` with the current configuration, that reads from
    /// an existing reader.
    ///
    /// # Example
    ///
    /// ```
    /// use pica::ReaderBuilder;
    /// use std::error::Error;
    /// use std::io::Cursor;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let cursor = Cursor::new(b"003@ \x1f0123456789\x1e");
    ///     let mut reader = ReaderBuilder::new().from_reader(cursor);
    ///     let record = reader.records().next().unwrap();
    ///     assert!(record.is_ok());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_reader<R: Read>(&self, reader: R) -> Reader<R> {
        Reader::new(self, reader)
    }

    /// Builds a new `Reader` with the current configuration, that reads from
    /// the given path, if some was provided, otherwise from `stdin`.
    ///
    /// # Example
    ///
    /// ```
    /// use pica::{ByteRecord, ReaderBuilder};
    /// use std::error::Error;
    /// use std::path::Path;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let mut reader = ReaderBuilder::new()
    ///         .from_path_or_stdin(Some("tests/data/119232022.dat.gz"))?;
    ///     let record = reader.records().next().unwrap();
    ///     assert!(record.is_ok());
    ///
    ///     let filename: Option<String> = None;
    ///     let mut reader = ReaderBuilder::new().from_path_or_stdin(filename)?;
    ///     assert!(reader.records().next().is_none());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_path_or_stdin<P: AsRef<Path>>(
        &self,
        path: Option<P>,
    ) -> Result<Reader<Box<dyn Read>>> {
        let reader: Box<dyn Read> = match path {
            None => Box::new(io::stdin()),
            Some(filename) => {
                let filename = filename.as_ref();

                if filename.extension() == Some(OsStr::new("gz")) {
                    Box::new(GzDecoder::new(File::open(filename)?))
                } else {
                    Box::new(File::open(filename)?)
                }
            }
        };

        Ok(Reader::new(self, reader))
    }

    /// Whether to skip invalid records or not.
    ///
    /// By default, if an invalid record occurs the reader moves forward until
    /// the next valid record was found or the end of file is reached. When
    /// this flag is disabled (`yes` is set to `false`), the iterator item wil
    /// be an `pica::Error`.
    ///
    /// # Example
    /// ```
    /// use pica::ReaderBuilder;
    /// use std::error::Error;
    /// use std::io::Cursor;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let data = b"003@ \x1f0123\x1e\n003@ \x1f!456\x1e\n003@ \x1f0789\x1e\n";
    ///
    ///     let mut reader = ReaderBuilder::new()
    ///         .skip_invalid(false)
    ///         .from_reader(Cursor::new(data));
    ///     let mut iter = reader.records();
    ///
    ///     assert!(iter.next().unwrap().is_ok());
    ///     assert!(iter.next().unwrap().is_err());
    ///     assert!(iter.next().unwrap().is_ok());
    ///     assert!(iter.next().is_none());
    ///
    ///     let mut reader = ReaderBuilder::new()
    ///         .skip_invalid(true)
    ///         .from_reader(Cursor::new(data));
    ///     let mut iter = reader.records();
    ///
    ///     assert!(iter.next().unwrap().is_ok());
    ///     assert!(iter.next().unwrap().is_ok());
    ///     assert!(iter.next().is_none());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn skip_invalid(mut self, yes: bool) -> Self {
        self.skip_invalid = yes;
        self
    }

    /// Change the inital capacity of an new `ByteRecord`.
    ///
    /// By default the inital capacity is set to `1024` bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use pica::ReaderBuilder;
    /// use std::error::Error;
    /// use std::io::Cursor;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let cursor = Cursor::new(b"003@ \x1f0123456789\x1e");
    ///     let mut reader =
    ///         ReaderBuilder::new().buffer_size(2048).from_reader(cursor);
    ///     let record = reader.records().next().unwrap();
    ///     assert!(record.is_ok());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }

    /// Change the limit of records to read.
    ///
    /// # Example
    ///
    /// ```
    /// use pica::ReaderBuilder;
    /// use std::error::Error;
    /// use std::io::Cursor;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let cursor =
    ///         Cursor::new(b"003@ \x1f0123456789\x1e\n003@ \x1f0123456789\x1e\n");
    ///
    ///     let mut reader = ReaderBuilder::new().limit(1).from_reader(cursor);
    ///     let mut records = reader.records();
    ///
    ///     let record = records.next().unwrap();
    ///     assert!(record.is_ok());
    ///
    ///     assert!(records.next().is_none());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn limit(mut self, buffer_size: usize) -> Self {
        self.limit = buffer_size;
        self
    }
}

/// A reader to read PICA+ records.
#[derive(Debug)]
pub struct Reader<R> {
    reader: BufReader<R>,
    skip_invalid: bool,
    buffer_size: usize,
    limit: usize,
}

impl<R: Read> Reader<R> {
    /// Create a new writer
    ///
    /// # Example
    ///
    /// ```
    /// use pica::{ByteRecord, Reader, ReaderBuilder, StringRecord};
    /// use std::error::Error;
    /// use std::io::Cursor;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let data = Cursor::new("003@ \x1f0123456789\x1e\n");
    ///     let mut reader = Reader::new(&ReaderBuilder::default(), data);
    ///     let records = reader
    ///         .records()
    ///         .map(Result::unwrap)
    ///         .collect::<Vec<StringRecord>>();
    ///
    ///     assert_eq!(
    ///         records,
    ///         vec![StringRecord::from_byte_record(ByteRecord::from_bytes(
    ///             "003@ \x1f0123456789\x1e\n".as_bytes()
    ///         )?)?]
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(builder: &ReaderBuilder, reader: R) -> Reader<R> {
        Self {
            reader: BufReader::with_capacity(builder.buffer_size, reader),
            skip_invalid: builder.skip_invalid,
            buffer_size: builder.buffer_size,
            limit: builder.limit,
        }
    }

    /// Returns an iterator over all `StringRecord`s.
    ///
    /// # Example
    ///
    /// ```
    /// use pica::ReaderBuilder;
    /// use std::error::Error;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let mut reader = ReaderBuilder::new()
    ///         .skip_invalid(true)
    ///         .from_path("tests/data/dump.dat.gz")?;
    ///     assert_eq!(reader.records().count(), 2);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn records(&mut self) -> StringRecordsIter<R> {
        StringRecordsIter::new(self)
    }

    /// Returns an iterator over all `ByteRecord`s.
    ///
    /// # Example
    ///
    /// ```
    /// use pica::ReaderBuilder;
    /// use std::error::Error;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let mut reader = ReaderBuilder::new()
    ///         .skip_invalid(true)
    ///         .from_path("tests/data/dump.dat.gz")?;
    ///     assert_eq!(reader.byte_records().count(), 2);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn byte_records(&mut self) -> ByteRecordsIter<R> {
        ByteRecordsIter::new(self)
    }
}

impl<R: Read> Deref for Reader<R> {
    type Target = BufReader<R>;

    fn deref(&self) -> &<Self as Deref>::Target {
        &self.reader
    }
}

impl<R: Read> DerefMut for Reader<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader
    }
}

/// An iterator that yields [`ByteRecord`]s.
///
/// Each iterator item is a [`Result<ByteRecord, ParsePicaError>`]. The caller
/// must handly possbile errors ([`io::Error`] or [`ParsePicaError`]). If the
/// `skip_invalid` flag is set, the iterator will ignore invalid records and
/// move forward to the next valid record.
pub struct ByteRecordsIter<'r, R: 'r> {
    reader: &'r mut Reader<R>,
    line: usize,
}

impl<'r, R: Read> ByteRecordsIter<'r, R> {
    fn new(reader: &'r mut Reader<R>) -> ByteRecordsIter<'r, R> {
        Self { reader, line: 0 }
    }
}

impl<'r, R: Read> Iterator for ByteRecordsIter<'r, R> {
    type Item = Result<ByteRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.limit > 0 && self.line >= self.reader.limit {
            return None;
        }

        let mut buffer: Vec<u8> = Vec::with_capacity(self.reader.buffer_size);

        match self.reader.read_until(b'\n', &mut buffer) {
            Err(e) => Some(Err(Error::from(e))),
            Ok(0) => None,
            Ok(_) => {
                let result = ByteRecord::from_bytes(buffer);
                self.line += 1;

                if result.is_err() && self.reader.skip_invalid {
                    // If the current item is invalid and the `skip_invalid`
                    // flag is set, we move forward to the next item until a
                    // valid item was found or the end of file is reached.
                    self.next()
                } else {
                    Some(result.map_err(|e| {
                        // Because the iterator tracks the current line, the
                        // error message of an invalid record can be enriched
                        // by the line number.
                        Error::InvalidRecord(ParsePicaError {
                            message: format!(
                                "Invalid record on line {}.",
                                self.line
                            ),
                            data: e.data,
                        })
                    }))
                }
            }
        }
    }
}

pub struct StringRecordsIter<'r, R: 'r> {
    iter: ByteRecordsIter<'r, R>,
    skip_invalid: bool,
}

impl<'r, R: Read> StringRecordsIter<'r, R> {
    fn new(reader: &'r mut Reader<R>) -> StringRecordsIter<'r, R> {
        let skip_invalid = reader.skip_invalid;
        let iter = ByteRecordsIter::new(reader);

        Self { iter, skip_invalid }
    }
}

impl<'r, R: Read> Iterator for StringRecordsIter<'r, R> {
    type Item = Result<StringRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Err(e)) => Some(Err(e)),
            Some(Ok(byte_record)) => {
                let result = StringRecord::from_byte_record(byte_record);
                if result.is_err() && self.skip_invalid {
                    self.next()
                } else {
                    Some(result)
                }
            }
            None => None,
        }
    }
}
