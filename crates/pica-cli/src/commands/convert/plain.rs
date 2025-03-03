use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufWriter, Write, stdout};

use bstr::ByteSlice;
use pica_record::prelude::*;

pub(crate) struct PlainWriter {
    writer: BufWriter<Box<dyn Write>>,
}

impl PlainWriter {
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

impl ByteRecordWrite for PlainWriter {
    fn write_byte_record(
        &mut self,
        record: &ByteRecord,
    ) -> std::io::Result<()> {
        for field in record.fields() {
            field.tag().write_to(&mut self.writer)?;
            if let Some(occurrence) = field.occurrence() {
                occurrence.write_to(&mut self.writer)?;
            }

            self.writer.write_all(b" ")?;

            for subfield in field.subfields() {
                self.writer
                    .write_all(&[b'$', subfield.code().as_byte()])?;
                self.writer.write_all(
                    &subfield.value().replace(b"$", b"$$"),
                )?;
            }

            self.writer.write_all(b"\n")?;
        }

        Ok(())
    }

    fn finish(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
