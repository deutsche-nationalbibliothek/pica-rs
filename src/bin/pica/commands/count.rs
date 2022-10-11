use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};

use clap::Parser;
use pica::{Reader, ReaderBuilder};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::skip_invalid_flag;
use crate::util::CliResult;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CountConfig {
    /// Skip invalid records that can't be decoded.
    pub(crate) skip_invalid: Option<bool>,
}

#[derive(Parser, Debug)]
pub(crate) struct Count {
    /// Skip invalid records that can't be decoded
    #[arg(short, long)]
    skip_invalid: bool,

    /// Append to the given file, do not overwrite
    #[arg(long)]
    append: bool,

    /// Prints only the number of records
    #[arg(long,
          conflicts_with_all = ["fields", "subfields", "csv", "tsv", "no_header"])]
    records: bool,

    /// Prints only the number of fields
    #[arg(long,
          conflicts_with_all = ["records", "subfields", "csv", "tsv", "no_header"])]
    fields: bool,

    /// Prints only the number of subfields
    #[arg(long,
          conflicts_with_all = ["records", "fields", "csv", "tsv", "no_header"])]
    subfields: bool,

    /// Write output comma-separated (CSV)
    #[arg(long, conflicts_with = "tsv")]
    csv: bool,

    /// Write output tab-separated (TSV)
    #[arg(long)]
    tsv: bool,

    /// Do not write header row
    #[arg(long)]
    no_header: bool,

    /// Write output to <OUTPUT> instead of stdout
    #[arg(short, long)]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format.
    #[arg(default_value = "-")]
    filenames: Vec<OsString>,
}

impl Count {
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.count,
            config.global
        );

        let mut writer: Box<dyn Write> = match self.output {
            Some(path) => Box::new(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(!self.append)
                    .append(self.append)
                    .open(path)?,
            ),
            None => Box::new(io::stdout()),
        };

        let mut records = 0;
        let mut fields = 0;
        let mut subfields = 0;

        for filename in self.filenames {
            let builder =
                ReaderBuilder::new().skip_invalid(skip_invalid);
            let mut reader: Reader<Box<dyn Read>> = match filename
                .to_str()
            {
                Some("-") => builder.from_reader(Box::new(io::stdin())),
                _ => builder.from_path(filename)?,
            };

            for result in reader.byte_records() {
                let record = result?;

                records += 1;
                fields += record.len();
                subfields += record
                    .iter()
                    .map(|field| field.len())
                    .sum::<usize>();
            }
        }

        if self.records {
            writeln!(writer, "{}", records)?;
        } else if self.fields {
            writeln!(writer, "{}", fields)?;
        } else if self.subfields {
            writeln!(writer, "{}", subfields)?;
        } else if self.csv {
            if !self.no_header {
                writeln!(writer, "records,fields,subfields")?;
            }
            writeln!(writer, "{},{},{}", records, fields, subfields)?;
        } else if self.tsv {
            if !self.no_header {
                writeln!(writer, "records\tfields\tsubfields")?;
            }
            writeln!(writer, "{}\t{}\t{}", records, fields, subfields)?;
        } else {
            writeln!(writer, "records: {}", records)?;
            writeln!(writer, "fields: {}", fields)?;
            writeln!(writer, "subfields: {}", subfields)?;
        }

        writer.flush()?;
        Ok(())
    }
}
