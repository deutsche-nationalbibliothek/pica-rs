use std::ffi::OsString;
use std::fs::File;

use csv;
use csv::Writer;
use pica_path::PathExt;
use pica_record::ByteRecord;

use super::Formatter;

#[derive(Debug)]
pub struct CsvFormatter {
    writer: Writer<File>,
}

impl CsvFormatter {
    pub fn new(output: OsString) -> Self {
        Self {
            writer: csv::Writer::from_path(output).unwrap(),
        }
    }
}

impl Formatter for CsvFormatter {
    fn fmt(
        &mut self,
        id: &str,
        record: &ByteRecord,
    ) -> std::io::Result<()> {
        self.writer
            .write_record(&[id.as_bytes(), record.idn().unwrap()])?;
        Ok(())
    }

    fn finish(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
