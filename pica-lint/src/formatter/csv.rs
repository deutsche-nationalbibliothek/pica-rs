use std::ffi::OsString;
use std::fs::File;

use bstr::BStr;
use csv::Writer;

use super::Formatter;
use crate::rules::Rule;

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
                b"level".to_vec(),
            ])
            .unwrap();

        Self { writer }
    }
}

impl Formatter for CsvFormatter {
    fn fmt(&mut self, rule: &Rule, idn: &BStr) -> std::io::Result<()> {
        self.writer.write_record([
            idn.as_ref(),
            rule.id.as_bytes(),
            rule.level.to_string().as_bytes(),
        ])?;

        Ok(())
    }

    fn finish(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
