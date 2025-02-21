use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use flate2::Compression;
use flate2::write::GzEncoder;

use crate::ByteRecord;

/// A trait that permits writing [ByteRecord]s.
pub trait ByteRecordWrite {
    /// Writes a [ByteRecord] into this writer.
    fn write_byte_record(
        &mut self,
        record: &ByteRecord,
    ) -> io::Result<()>;

    /// Finish the underlying writer.
    fn finish(&mut self) -> io::Result<()>;
}

/// Configures and build a [ByteRecord] writer.
#[derive(Default)]
pub struct WriterBuilder {
    append: bool,
    gzip: bool,
}

impl WriterBuilder {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a [ByteRecord] writer from this configuration that writes
    /// to the given path.
    pub fn from_path<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> io::Result<Box<dyn ByteRecordWrite>> {
        let path = path.as_ref();
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(!self.append)
            .append(self.append)
            .open(path)?;

        if self.gzip
            || path.extension().and_then(OsStr::to_str) == Some("gz")
        {
            Ok(Box::new(GzipWriter::new(file)))
        } else {
            Ok(Box::new(PlainWriter::new(file)))
        }
    }

    /// Builds a [ByteRecord] writer from this configuration that writes
    /// to the given path, if given, otherwise write to `stdout`.
    pub fn from_path_or_stdout<P: AsRef<Path>>(
        &self,
        path: Option<P>,
    ) -> io::Result<Box<dyn ByteRecordWrite>> {
        match path {
            Some(path) => self.from_path(path),
            None => {
                if self.gzip {
                    Ok(Box::new(GzipWriter::new(Box::new(
                        io::stdout().lock(),
                    ))))
                } else {
                    Ok(Box::new(PlainWriter::new(Box::new(
                        io::stdout(),
                    ))))
                }
            }
        }
    }

    /// Whether to use a gzip encoder or not.
    ///
    /// When this flag is set, the writer encode the records in gzip
    /// format. This flag is disabled by default and has no effect when
    /// writing to `stdout`.
    ///
    /// # Panics
    ///
    /// It's an error to use this flag in append-mode.
    pub fn gzip(mut self, yes: bool) -> Self {
        assert!(!yes || (yes ^ self.append));
        self.gzip = yes;
        self
    }

    /// Whether to append to a given file or not.
    ///
    /// When this flag is set, the writer appends to the given file. If
    /// the file does not exists, the file is created. This flag has
    /// no effect when writing to `stdout`. This option is disabled by
    /// default.
    ///
    /// # Panics
    ///
    /// It's an error to use this flag in combination with a gzip
    /// writer.
    pub fn append(mut self, yes: bool) -> Self {
        assert!(!yes || (yes ^ self.gzip));
        self.append = yes;
        self
    }
}

/// A plain buffered [ByteRecord] writer.
pub struct PlainWriter<W: Write>(BufWriter<W>);

impl<W: Write> PlainWriter<W> {
    /// Creates a new [PlainWriter].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::io::PlainWriter;
    /// use pica_record::prelude::*;
    ///
    /// let mut file = tempfile::tempfile()?;
    /// let mut writer = PlainWriter::new(file);
    /// let record = ByteRecord::from_bytes(b"003@ \x1f0123456789X\x1e\n")?;
    /// writer.write_byte_record(&record)?;
    /// writer.finish()?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(inner: W) -> Self {
        Self(BufWriter::new(inner))
    }
}

impl<W: Write> ByteRecordWrite for PlainWriter<W> {
    /// Writes a [ByteRecord] to the [PlainWriter].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    ///
    /// let mut file = tempfile::NamedTempFile::new()?;
    /// let mut writer = WriterBuilder::new().from_path(file)?;
    /// let record = ByteRecord::from_bytes(b"003@ \x1f0123456789X\x1e\n")?;
    /// writer.write_byte_record(&record)?;
    /// writer.finish()?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    fn write_byte_record(
        &mut self,
        record: &ByteRecord,
    ) -> io::Result<()> {
        record.write_to(&mut self.0)
    }

    /// Finish the [PlainWriter].
    #[inline(always)]
    fn finish(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

/// A [ByteRecord] writer that gzip encodes records.
pub struct GzipWriter<W: Write>(GzEncoder<W>);

impl<W: Write> GzipWriter<W> {
    /// Creates a new [GzipWriter].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::io::GzipWriter;
    /// use pica_record::prelude::*;
    ///
    /// let mut file = tempfile::tempfile()?;
    /// let mut writer = GzipWriter::new(file);
    /// let record = ByteRecord::from_bytes(b"003@ \x1f0123456789X\x1e\n")?;
    /// writer.write_byte_record(&record)?;
    /// writer.finish()?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(inner: W) -> GzipWriter<W> {
        Self(GzEncoder::new(inner, Compression::default()))
    }
}

impl<W: Write> ByteRecordWrite for GzipWriter<W> {
    /// Writes a [ByteRecord] to the [GzipWriter].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    ///
    /// let mut file = tempfile::NamedTempFile::with_suffix("gz")?;
    /// let mut writer = WriterBuilder::new().gzip(true).from_path(file)?;
    /// let record = ByteRecord::from_bytes(b"003@ \x1f0123456789X\x1e\n")?;
    /// writer.write_byte_record(&record)?;
    /// writer.finish()?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    fn write_byte_record(
        &mut self,
        record: &ByteRecord,
    ) -> io::Result<()> {
        record.write_to(&mut self.0)
    }

    /// Finish the [GzipWriter].
    #[inline(always)]
    fn finish(&mut self) -> io::Result<()> {
        self.0.try_finish()?;
        Ok(())
    }
}
