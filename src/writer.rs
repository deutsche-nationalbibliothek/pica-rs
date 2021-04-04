//! PICA+ writer
//!
//! TODO: Writer::into_inner() -> Test from_reader

use crate::error::Result;
use crate::ByteRecord;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::ops::{Deref, DerefMut};
use std::path::Path;

/// Configures and builds a PICA+ writer.
#[derive(Debug)]
pub struct WriterBuilder {
    buffer_size: usize,
}

impl Default for WriterBuilder {
    fn default() -> WriterBuilder {
        WriterBuilder {
            buffer_size: 65_536,
        }
    }
}

impl WriterBuilder {
    /// Create a new `WriterBuilder` for writing PICA+ records.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, WriterBuilder};
    /// # use pica::{ReaderBuilder, StringRecord};
    /// use std::error::Error;
    /// use tempfile::Builder;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let record =
    ///         ByteRecord::from_bytes("003@ \x1f0123456789\x1e\n".as_bytes())?;
    ///
    ///     let mut tempfile = Builder::new().rand_bytes(5).tempfile()?;
    ///     let path = tempfile.path().to_owned();
    ///
    ///     let mut writer = WriterBuilder::new().from_writer(tempfile);
    ///     writer.write_byte_record(&record)?;
    ///     writer.flush()?;
    ///     #
    ///     # let mut reader = ReaderBuilder::new().from_path(path)?;
    ///     # assert_eq!(reader.records().next().unwrap()?,
    ///     #     StringRecord::from_byte_record(record)?);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new() -> WriterBuilder {
        WriterBuilder::default()
    }

    /// Builds a new `Writer` with the current configuration, that writes to
    /// the specified file path.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, WriterBuilder};
    /// # use pica::{ReaderBuilder, StringRecord};
    /// use std::error::Error;
    /// use tempfile::Builder;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let record =
    ///         ByteRecord::from_bytes("003@ \x1f0123456789\x1e\n".as_bytes())?;
    ///
    ///     let mut tempfile = Builder::new().rand_bytes(5).tempfile()?;
    ///
    ///     let mut writer = WriterBuilder::new().from_path(&tempfile.path())?;
    ///     writer.write_byte_record(&record)?;
    ///     writer.flush()?;
    ///     #
    ///     # let mut reader = ReaderBuilder::new().from_path(&tempfile.path())?;
    ///     # assert_eq!(reader.records().next().unwrap()?,
    ///     #     StringRecord::from_byte_record(record)?);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_path<P: AsRef<Path>>(&self, path: P) -> Result<Writer<File>> {
        Ok(Writer::new(self, File::create(path)?))
    }

    /// Builds a new `Writer` with the current configuration, that writes to an
    /// existing writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, WriterBuilder};
    /// # use pica::{ReaderBuilder, StringRecord};
    /// use std::error::Error;
    /// use tempfile::Builder;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let record =
    ///         ByteRecord::from_bytes("003@ \x1f0123456789\x1e\n".as_bytes())?;
    ///
    ///     let mut tempfile = Builder::new().tempfile()?;
    ///     # let filename = tempfile.path().to_owned();
    ///
    ///     let mut writer = WriterBuilder::new().from_writer(tempfile);
    ///     writer.write_byte_record(&record)?;
    ///     writer.flush()?;
    ///
    ///     #
    ///     # let mut reader = ReaderBuilder::new().from_path(&filename)?;
    ///     # assert_eq!(reader.records().next().unwrap()?,
    ///     #     StringRecord::from_byte_record(record)?);
    ///     Ok(())
    /// }
    /// ```
    pub fn from_writer<W: Write>(&self, writer: W) -> Writer<W> {
        Writer::new(self, writer)
    }

    /// Builds a new `Writer` with the current configuration, that writes to
    /// the specified file path, if some was provided, otherwise to `stdout`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, WriterBuilder};
    /// use std::error::Error;
    /// use tempfile::Builder;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let record =
    ///         ByteRecord::from_bytes("003@ \x1f0123456789\x1e\n".as_bytes())?;
    ///
    ///     let mut tempfile = Builder::new().rand_bytes(5).tempfile()?;
    ///     let mut writer =
    ///         WriterBuilder::new().from_path_or_stdout(Some(&tempfile.path()));
    ///     assert!(writer.is_ok());
    ///
    ///     let mut writer =
    ///         WriterBuilder::new().from_path_or_stdout(None::<String>);
    ///     assert!(writer.is_ok());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_path_or_stdout<P: AsRef<Path>>(
        &self,
        path: Option<P>,
    ) -> Result<Writer<Box<dyn Write>>> {
        let writer: Box<dyn Write> = match path {
            Some(path) => Box::new(File::create(path)?),
            None => Box::new(io::stdout()),
        };

        Ok(Writer::new(self, writer))
    }
}

/// A writer to write PICA+ records.
#[derive(Debug)]
pub struct Writer<W: Write> {
    inner: BufWriter<W>,
}

impl<W: Write> Deref for Writer<W> {
    type Target = BufWriter<W>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<W: Write> DerefMut for Writer<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<W: Write> Writer<W> {
    /// Creates a new writer
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, Writer, WriterBuilder};
    /// # use pica::{ReaderBuilder, StringRecord};
    /// use std::error::Error;
    /// use tempfile::Builder;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let record =
    ///         ByteRecord::from_bytes("003@ \x1f0123456789\x1e\n".as_bytes())?;
    ///
    ///     let mut tempfile = Builder::new().tempfile()?;
    ///     # let filename = tempfile.path().to_owned();
    ///
    ///     let mut writer = Writer::new(&WriterBuilder::default(), tempfile);
    ///     writer.write_byte_record(&record)?;
    ///     writer.flush()?;
    ///
    ///     #
    ///     # let mut reader = ReaderBuilder::new().from_path(&filename)?;
    ///     # assert_eq!(reader.records().next().unwrap()?,
    ///     #     StringRecord::from_byte_record(record)?);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(builder: &WriterBuilder, inner: W) -> Writer<W> {
        Self {
            inner: BufWriter::with_capacity(builder.buffer_size, inner),
        }
    }

    /// Write a byte record into this writer
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, Writer, WriterBuilder};
    /// # use pica::{ReaderBuilder, StringRecord};
    /// use std::error::Error;
    /// use tempfile::Builder;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let record =
    ///         ByteRecord::from_bytes("003@ \x1f0123456789\x1e\n".as_bytes())?;
    ///
    ///     let mut tempfile = Builder::new().tempfile()?;
    ///     # let filename = tempfile.path().to_owned();
    ///
    ///     let mut writer = Writer::new(&WriterBuilder::default(), tempfile);
    ///     writer.write_byte_record(&record)?;
    ///     writer.flush()?;
    ///
    ///     #
    ///     # let mut reader = ReaderBuilder::new().from_path(&filename)?;
    ///     # assert_eq!(reader.records().next().unwrap()?,
    ///     #     StringRecord::from_byte_record(record)?);
    ///     Ok(())
    /// }
    /// ```
    pub fn write_byte_record(&mut self, record: &ByteRecord) -> Result<()> {
        if let Some(raw_data) = &record.raw_data {
            self.inner.write_all(raw_data)?;
        } else {
            record.write(self)?;
        }

        Ok(())
    }

    /// Flushes the underlying writer.
    ///
    /// If an problem occurs when writing to the underlying writer, an error is
    /// returned.
    pub fn flush(&mut self) -> Result<()> {
        self.inner.flush()?;
        Ok(())
    }
}
