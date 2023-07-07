use std::io::{BufWriter, Write};
use std::ops::{Deref, DerefMut};

use crate::error::Result;
use crate::ByteRecord;

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
    /// use pica::{ByteRecord, PicaWriter, PlainWriter, WriterBuilder};
    /// # use pica::{ReaderBuilder, StringRecord};
    /// use std::error::Error;
    /// use tempfile::Builder;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let record = ByteRecord::from_bytes("003@ \x1f0123456789\x1e\n".as_bytes())?;
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
    /// If an problem occurs when writing to the underlying writer, an
    /// error is returned.
    fn finish(&mut self) -> Result<()> {
        self.inner.flush()?;
        Ok(())
    }
}
