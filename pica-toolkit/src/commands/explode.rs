use std::ffi::OsString;

use clap::Parser;
use pica_record::io::{ReaderBuilder, RecordsIterator, WriterBuilder};
use pica_record::{ByteRecord, Level};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::skip_invalid_flag;
use crate::util::CliResult;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ExplodeConfig {
    /// Skip invalid records that can't be decoded.
    pub(crate) skip_invalid: Option<bool>,
}

#[derive(Parser, Debug)]
pub(crate) struct Explode {
    /// Skip invalid records that can't be decoded.
    #[arg(short, long)]
    skip_invalid: bool,

    /// Write output to <OUTPUT> instead of stdout
    #[arg(short, long)]
    output: Option<OsString>,

    /// Split a record by level (main, local, copy).
    level: Level,

    /// Read one or more files in normalized PICA+ format. If no
    /// filenames where given or a filename is "-", data is read from
    /// standard input (stdin)
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

macro_rules! record_bytes {
    ($fields:expr) => {{
        let mut buffer = Vec::<u8>::new();
        $fields.iter().for_each(|field| {
            let _ = field.write_to(&mut buffer);
        });
        buffer.push(b'\n');
        buffer
    }};
}

macro_rules! push_record {
    ($records:expr, $main:expr, $local:expr, $acc:expr) => {
        if !$acc.is_empty() {
            let mut record = $main.clone();
            if let Some(local) = $local {
                record.push(local);
            }
            record.extend_from_slice(&$acc);

            $records.push(record);
            $acc.clear();
        }
    };

    ($records:expr, $main:expr, $acc:expr) => {
        if !$acc.is_empty() {
            let mut record = $main.clone();
            record.extend_from_slice(&$acc);
            $records.push(record);
            $acc.clear();
        }
    };
}

impl Explode {
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.explode,
            config.global
        );

        let mut writer =
            WriterBuilder::new().from_path_or_stdout(self.output)?;

        for filename in self.filenames {
            let mut reader =
                ReaderBuilder::new().from_path(filename)?;

            while let Some(result) = reader.next() {
                match result {
                    Err(e) => {
                        if e.is_invalid_record() && skip_invalid {
                            continue;
                        } else {
                            return Err(e.into());
                        }
                    }
                    Ok(record) => match self.level {
                        Level::Main => {
                            writer.write_byte_record(&record)?
                        }
                        Level::Copy => {
                            let mut main = vec![];
                            let mut acc = vec![];
                            let mut records = vec![];
                            let mut local = None;
                            let mut count = None;

                            for field in record.iter() {
                                match field.level() {
                                    Level::Main => main.push(field),
                                    Level::Local => {
                                        push_record!(
                                            records, main, local, acc
                                        );

                                        local = Some(field);
                                        count = None;
                                    }
                                    Level::Copy => {
                                        if count != field.occurrence() {
                                            push_record!(
                                                records, main, local,
                                                acc
                                            );

                                            count = field.occurrence();
                                        }

                                        acc.push(field);
                                    }
                                }
                            }

                            push_record!(records, main, local, acc);

                            for fields in records {
                                let data = record_bytes!(fields);
                                let record =
                                    ByteRecord::from_bytes(&data)
                                        .expect("valid record");
                                writer.write_byte_record(&record)?;
                            }
                        }
                        Level::Local => {
                            let mut main = vec![];
                            let mut acc = vec![];
                            let mut records = vec![];

                            for field in record.iter() {
                                match field.level() {
                                    Level::Main => main.push(field),
                                    Level::Copy => acc.push(field),
                                    Level::Local => {
                                        push_record!(
                                            records, main, acc
                                        );
                                        acc.push(field)
                                    }
                                }
                            }

                            push_record!(records, main, acc);

                            for fields in records.iter() {
                                let data = record_bytes!(fields);
                                let record =
                                    ByteRecord::from_bytes(&data)
                                        .unwrap();
                                writer.write_byte_record(&record)?;
                            }
                        }
                    },
                }
            }
        }

        writer.finish()?;
        Ok(())
    }
}
