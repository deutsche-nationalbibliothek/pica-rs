use std::ffi::OsString;
use std::fs::File;

use csv;
use csv::Writer;
use pica_path::PathExt;
use pica_record::ByteRecord;

use super::Formatter;
use crate::rules::Severity;

#[derive(Debug)]
pub struct CsvFormatter {
    writer: Writer<File>,
}

impl CsvFormatter {
    pub fn new(output: OsString) -> Self {
        let mut writer = csv::Writer::from_path(output).unwrap();
        writer
            .write_record(&[
                b"idn".to_vec(),
                b"rule".to_vec(),
                b"severity".to_vec(),
            ])
            .unwrap();

        Self { writer }
    }
}

impl Formatter for CsvFormatter {
    fn fmt(
        &mut self,
        id: &str,
        record: &ByteRecord,
        severity: &Severity,
    ) -> std::io::Result<()> {
        self.writer.write_record(&[
            record.idn().unwrap(),
            id.as_bytes(),
            severity.to_string().as_bytes(),
        ])?;
        Ok(())
    }

    fn finish(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
