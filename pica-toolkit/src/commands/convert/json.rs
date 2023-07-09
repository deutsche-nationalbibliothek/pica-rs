use std::ffi::OsString;
use std::fs::File;
use std::io::{self, stdout, BufWriter, Write};

use pica_record::io::ByteRecordWrite;
use pica_record::ByteRecord;
use serde_json::Value;

pub(crate) struct JsonWriter {
    writer: BufWriter<Box<dyn Write>>,
    count: usize,
}

impl JsonWriter {
    pub(crate) fn new(output: Option<OsString>) -> io::Result<Self> {
        let mut writer: BufWriter<Box<dyn Write>> =
            if let Some(filename) = output {
                BufWriter::new(Box::new(File::create(filename)?))
            } else {
                BufWriter::new(Box::new(stdout()))
            };

        writer.write_all(&[b'['])?;
        Ok(Self { writer, count: 0 })
    }
}

impl ByteRecordWrite for JsonWriter {
    fn write_byte_record(
        &mut self,
        record: &ByteRecord,
    ) -> std::io::Result<()> {
        let mut fields: Vec<Value> = Vec::new();

        for field in record.iter() {
            let mut data: Vec<serde_json::Value> = Vec::new();
            data.push(serde_json::Value::String(
                field.tag().to_string(),
            ));

            if let Some(occurence) = field.occurrence() {
                data.push(serde_json::Value::String(
                    occurence.to_string(),
                ));
            } else {
                data.push(serde_json::Value::Null);
            }

            for subfield in field.subfields() {
                data.push(serde_json::Value::String(
                    subfield.code().to_string(),
                ));

                data.push(serde_json::Value::String(
                    subfield.value().to_string(),
                ));
            }

            fields.push(Value::Array(data));
        }

        let data = serde_json::Value::Array(fields);
        if self.count > 0 {
            write!(self.writer, ",{}", data)?;
        } else {
            write!(self.writer, "{}", data)?;
        }

        self.count += 1;
        Ok(())
    }

    fn finish(&mut self) -> io::Result<()> {
        self.writer.write_all(&[b']'])?;
        self.writer.flush()
    }
}
