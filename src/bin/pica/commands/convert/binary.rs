use std::ffi::OsString;
use std::fs::File;
use std::io::{self, stdin, stdout, BufReader, BufWriter, Read, Write};

use pica_record::io::{
    ByteRecordWrite, ReadPicaError, RecordsIterator,
};
use pica_record::ByteRecord;

pub(crate) struct BinaryWriter {
    writer: BufWriter<Box<dyn Write>>,
}

impl BinaryWriter {
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

pub(crate) struct BinaryReader {
    inner: BufReader<Box<dyn Read>>,
}

impl BinaryReader {
    pub(crate) fn new(input: Option<OsString>) -> io::Result<Self> {
        let inner: BufReader<Box<dyn Read>> =
            if let Some(filename) = input {
                BufReader::new(Box::new(File::open(filename)?))
            } else {
                BufReader::new(Box::new(stdin()))
            };

        Ok(Self { inner })
    }
}

impl RecordsIterator for BinaryReader {
    type Item<'a> = Result<ByteRecord<'a>, ReadPicaError> where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        todo!()
    }
}

impl ByteRecordWrite for BinaryWriter {
    fn write_byte_record(
        &mut self,
        record: &ByteRecord,
    ) -> std::io::Result<()> {
        for field in record.iter() {
            self.writer.write_all(field.tag())?;
            if let Some(occurrence) = field.occurrence() {
                occurrence.write_to(&mut self.writer)?;
            }

            self.writer.write_all(&[b' '])?;
            for subfield in field.subfields() {
                subfield.write_to(&mut self.writer)?;
            }

            self.writer.write_all(&[b'\x1e'])?;
        }

        self.writer.write_all(&[b'\x1d'])?;
        Ok(())
    }

    fn finish(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
