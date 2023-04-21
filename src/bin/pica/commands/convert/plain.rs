use std::ffi::OsString;
use std::fs::File;
use std::io::{self, stdout, BufWriter, Write};

use bstr::ByteSlice;
use pica_record::io::ByteRecordWrite;
use pica_record::ByteRecord;

pub(crate) struct PlainWriter {
    writer: BufWriter<Box<dyn Write>>,
}

impl PlainWriter {
    pub(crate) fn new(output: Option<OsString>) -> io::Result<Self> {
        let writer: BufWriter<Box<dyn Write>> =
            if let Some(filename) = output {
                BufWriter::new(Box::new(File::open(filename)?))
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
        for field in record.iter() {
            self.writer.write_all(field.tag())?;
            if let Some(occurrence) = field.occurrence() {
                occurrence.write_to(&mut self.writer)?;
            }

            self.writer.write(&[b' '])?;

            for subfield in field.subfields() {
                self.writer.write(&[b'$', subfield.code() as u8])?;
                self.writer
                    .write(&subfield.value().replace(b"$", b"$$"))?;
            }

            self.writer.write(&[b'\n'])?;
        }

        Ok(())
    }

    fn finish(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
