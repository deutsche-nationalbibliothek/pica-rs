use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufWriter, Write, stdout};

use pica_record::prelude::*;

pub(crate) struct ImportWriter {
    writer: BufWriter<Box<dyn Write>>,
}

impl ImportWriter {
    pub(crate) fn new(output: Option<OsString>) -> io::Result<Self> {
        let writer: BufWriter<Box<dyn Write>> =
            if let Some(filename) = output {
                BufWriter::new(Box::new(File::create(filename)?))
            } else {
                BufWriter::new(Box::new(stdout()))
            };

        Ok(Self { writer })
    }
}

impl ByteRecordWrite for ImportWriter {
    fn write_byte_record(
        &mut self,
        record: &ByteRecord,
    ) -> std::io::Result<()> {
        self.writer.write_all(b"'\x1d\x0a")?;

        for field in record.fields() {
            self.writer.write_all(b"\x1e")?;

            field.tag().write_to(&mut self.writer)?;
            if let Some(occurrence) = field.occurrence() {
                occurrence.write_to(&mut self.writer)?;
            }

            self.writer.write_all(b" ")?;
            for subfield in field.subfields() {
                subfield.write_to(&mut self.writer)?;
            }

            self.writer.write_all(b"\x0a")?;
        }

        Ok(())
    }

    fn finish(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
