use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Result, Write};
use std::path::Path;

use flate2::write::GzEncoder;
use flate2::Compression;

use crate::ByteRecord;

/// A tait that permits writing [ByteRecord]s.
pub trait ByteRecordWrite {
    fn write_byte_record(&mut self, record: &ByteRecord) -> Result<()>;
    fn finish(&mut self) -> Result<()>;
}

/// Configures and build a [ByteRecord] writer.
pub struct WriterBuilder {
    append: bool,
    gzip: bool,
}

impl Default for WriterBuilder {
    fn default() -> Self {
        Self {
            append: false,
            gzip: false,
        }
    }
}

impl WriterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_path<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Box<dyn ByteRecordWrite>> {
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

    pub fn from_path_or_stdout<P: AsRef<Path>>(
        &self,
        path: Option<P>,
    ) -> Result<Box<dyn ByteRecordWrite>> {
        match path {
            Some(path) => self.from_path(path),
            None => {
                Ok(Box::new(PlainWriter::new(Box::new(io::stdout()))))
            }
        }
    }

    pub fn gzip(mut self, yes: bool) -> Self {
        self.gzip = yes;
        self
    }

    pub fn append(mut self, yes: bool) -> Self {
        self.append = yes;
        self
    }
}

/// A plain buffered [ByteRecord] writer.
pub struct PlainWriter<W: Write> {
    inner: BufWriter<W>,
}

impl<W: Write> PlainWriter<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner: BufWriter::new(inner),
        }
    }
}

impl<W: Write> ByteRecordWrite for PlainWriter<W> {
    fn write_byte_record(&mut self, record: &ByteRecord) -> Result<()> {
        record.write_to(&mut self.inner)
    }

    fn finish(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

/// A [ByteRecord] writer that gzip encodes records.
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

impl<W: Write> ByteRecordWrite for GzipWriter<W> {
    fn write_byte_record(&mut self, record: &ByteRecord) -> Result<()> {
        record.write_to(&mut self.inner)
    }

    fn finish(&mut self) -> Result<()> {
        self.inner.try_finish()?;
        Ok(())
    }
}
