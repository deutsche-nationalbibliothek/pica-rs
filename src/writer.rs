use crate::error::Result;
use crate::ByteRecord;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::ops::{Deref, DerefMut};
use std::path::Path;

use flate2::write::GzEncoder;
use flate2::Compression;

/// Configures and builds a PICA+ writer.
#[derive(Debug)]
pub struct WriterBuilder {
    buffer_size: usize,
    gzip: bool,
}

impl Default for WriterBuilder {
    fn default() -> WriterBuilder {
        WriterBuilder {
            buffer_size: 65_536,
            gzip: false,
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
    pub fn from_path<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Box<dyn PicaWriter>> {
        let path = path.as_ref();

        if self.gzip || path.extension() == Some(OsStr::new("gz")) {
            Ok(Box::new(GzipWriter::new(File::create(path)?)))
        } else {
            Ok(Box::new(PlainWriter::new(self, File::create(path)?)))
        }
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
    pub fn from_writer<W: Write + 'static>(
        &self,
        writer: W,
    ) -> Box<dyn PicaWriter> {
        if self.gzip {
            Box::new(GzipWriter::new(writer))
        } else {
            Box::new(PlainWriter::new(self, writer))
        }
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
    ) -> Result<Box<dyn PicaWriter>> {
        if let Some(path) = path {
            let path = path.as_ref();

            if self.gzip || path.extension() == Some(OsStr::new("gz")) {
                Ok(Box::new(GzipWriter::new(File::create(path)?)))
            } else {
                Ok(Box::new(PlainWriter::new(self, File::create(path)?)))
            }
        } else {
            Ok(Box::new(PlainWriter::new(self, Box::new(io::stdout()))))
        }
    }

    pub fn gzip(mut self, yes: bool) -> Self {
        self.gzip = yes;
        self
    }
}

pub trait PicaWriter: Write {
    fn write_byte_record(&mut self, record: &ByteRecord) -> Result<()>;
    fn finish(&mut self) -> Result<()>;
}

/// A writer to write PICA+ records.
#[derive(Debug)]
pub struct PlainWriter<W: Write> {
    inner: BufWriter<W>,
}

impl<W: Write> Deref for PlainWriter<W> {
    type Target = BufWriter<W>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<W: Write> DerefMut for PlainWriter<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<W: Write> PlainWriter<W> {
    /// Creates a new writer
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, PlainWriter, PicaWriter, WriterBuilder};
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
    ///     let mut writer = PlainWriter::new(&WriterBuilder::default(), tempfile);
    ///     writer.write_byte_record(&record)?;
    ///     writer.finish()?;
    ///
    ///     #
    ///     # let mut reader = ReaderBuilder::new().from_path(&filename)?;
    ///     # assert_eq!(reader.records().next().unwrap()?,
    ///     #     StringRecord::from_byte_record(record)?);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(builder: &WriterBuilder, inner: W) -> PlainWriter<W> {
        Self {
            inner: BufWriter::with_capacity(builder.buffer_size, inner),
        }
    }
}

impl<W: Write> Write for PlainWriter<W> {
    #[inline]
    fn write(
        &mut self,
        buf: &[u8],
    ) -> std::result::Result<usize, std::io::Error> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
        self.inner.flush()
    }
}

impl<W: Write> PicaWriter for PlainWriter<W> {
    /// Write a byte record into this writer
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, PlainWriter, PicaWriter, WriterBuilder};
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
    ///     let mut writer = PlainWriter::new(&WriterBuilder::default(), tempfile);
    ///     writer.write_byte_record(&record)?;
    ///     writer.finish()?;
    ///
    ///     #
    ///     # let mut reader = ReaderBuilder::new().from_path(&filename)?;
    ///     # assert_eq!(reader.records().next().unwrap()?,
    ///     #     StringRecord::from_byte_record(record)?);
    ///     Ok(())
    /// }
    /// ```
    fn write_byte_record(&mut self, record: &ByteRecord) -> Result<()> {
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
    fn finish(&mut self) -> Result<()> {
        self.inner.flush()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct GzipWriter<W: Write> {
    inner: GzEncoder<W>,
}

impl<W: Write> GzipWriter<W> {
    pub fn new(inner: W) -> GzipWriter<W> {
        Self {
            inner: GzEncoder::new(inner, Compression::default()),
        }
    }
}

impl<W: Write> Write for GzipWriter<W> {
    #[inline]
    fn write(
        &mut self,
        buf: &[u8],
    ) -> std::result::Result<usize, std::io::Error> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
        self.inner.flush()
    }
}

impl<W: Write> PicaWriter for GzipWriter<W> {
    fn write_byte_record(&mut self, record: &ByteRecord) -> Result<()> {
        if let Some(raw_data) = &record.raw_data {
            self.inner.write_all(raw_data)?;
        } else {
            record.write(self)?;
        }

        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        self.inner.try_finish()?;
        Ok(())
    }
}
