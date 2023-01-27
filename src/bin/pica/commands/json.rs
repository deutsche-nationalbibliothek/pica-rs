use std::ffi::OsString;
use std::io::{self, Read, Write};

use clap::Parser;
use pica::{PicaWriter, Reader, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::skip_invalid_flag;
use crate::translit::translit_maybe;
use crate::util::CliResult;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct JsonConfig {
    pub(crate) skip_invalid: Option<bool>,
}

#[derive(Parser, Debug)]
pub(crate) struct Json {
    /// Skip invalid records that can't be decoded
    #[arg(short, long)]
    skip_invalid: bool,

    /// Limit the result to first <n> records
    #[arg(long, short = 'n', value_name = "n", default_value = "0")]
    limit: usize,

    /// Transliterate output into the selected normalform <NF>
    /// (possible values: "nfd", "nfkd", "nfc" and "nfkc")
    #[arg(long,
          value_name = "NF", 
          value_parser = ["nfd", "nfkd", "nfc", "nfkc"],
          hide_possible_values = true,
    )]
    translit: Option<String>,

    /// Write output to <filename> instead of stdout
    #[arg(short, long, value_name = "filename")]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format.
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Json {
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.json,
            config.global
        );

        let mut writer: Box<dyn PicaWriter> =
            WriterBuilder::new().from_path_or_stdout(self.output)?;
        writer.write_all(b"[")?;

        for filename in self.filenames {
            let builder = ReaderBuilder::new()
                .skip_invalid(skip_invalid)
                .limit(self.limit);

            let mut reader: Reader<Box<dyn Read>> = match filename
                .to_str()
            {
                Some("-") => builder.from_reader(Box::new(io::stdin())),
                _ => builder.from_path(filename)?,
            };

            for (count, result) in reader.records().enumerate() {
                let record = result?;

                if count > 0 {
                    writer.write_all(b",")?;
                }

                let j = translit_maybe(
                    &serde_json::to_string(&record).unwrap(),
                    self.translit.as_deref(),
                );

                writer.write_all(j.as_bytes())?;
            }
        }

        writer.write_all(b"]")?;
        writer.flush()?;

        Ok(())
    }
}
