use std::ffi::OsString;
use std::fs::File;
use std::io::{stdout, IsTerminal, Write};
use std::str::FromStr;

use bstr::ByteSlice;
use clap::Parser;
use pica_record::io::{ReaderBuilder, RecordsIterator};
use serde::{Deserialize, Serialize};
use termcolor::{
    Color, ColorChoice, ColorSpec, NoColor, StandardStream, WriteColor,
};

use crate::config::Config;
use crate::progress::Progress;
use crate::skip_invalid_flag;
use crate::translit::translit_maybe;
use crate::util::{CliError, CliResult};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub(crate) struct PrintConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) tag_color: Option<PrintColorSpec>,
    pub(crate) occurrence_color: Option<PrintColorSpec>,
    pub(crate) code_color: Option<PrintColorSpec>,
    pub(crate) value_color: Option<PrintColorSpec>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub(crate) struct PrintColorSpec {
    pub(crate) color: Option<String>,
    #[serde(default)]
    pub(crate) bold: bool,
    #[serde(default)]
    pub(crate) italic: bool,
    #[serde(default)]
    pub(crate) underline: bool,
    #[serde(default)]
    pub(crate) intense: bool,
    #[serde(default)]
    pub(crate) dimmed: bool,
}

impl TryFrom<&PrintColorSpec> for ColorSpec {
    type Error = CliError;

    fn try_from(value: &PrintColorSpec) -> Result<Self, Self::Error> {
        let fg_color = if let Some(fg_color_str) = &value.color {
            if let Ok(c) = Color::from_str(fg_color_str) {
                Some(c)
            } else {
                return Err(CliError::Other(format!(
                    "invalid color '{fg_color_str}'"
                )));
            }
        } else {
            None
        };

        Ok(ColorSpec::new()
            .set_fg(fg_color)
            .set_bold(value.bold)
            .set_italic(value.italic)
            .set_underline(value.underline)
            .set_intense(value.intense)
            .set_dimmed(value.dimmed)
            .clone())
    }
}

/// Print records in human readable format
#[derive(Parser, Debug)]
pub(crate) struct Print {
    /// Skip invalid records that can't be decoded
    #[arg(short, long)]
    skip_invalid: bool,

    /// Limit the result to first <n> records
    #[arg(long, short, value_name = "n", default_value = "0")]
    limit: usize,

    /// Transliterate output into the selected normal form <NF>
    #[arg(long,
          value_name = "NF",
          value_parser = ["nfd", "nfkd", "nfc", "nfkc"]
    )]
    translit: Option<String>,

    /// Specify color settings for use in the output
    #[arg(long,
          value_parser = ["auto", "always", "ansi", "never"],
          default_value = "auto",
    )]
    color: String,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// Write output to <filename> instead of stdout
    #[arg(short, long, value_name = "filename")]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format.
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Print {
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.print,
            config.global
        );

        let choice = match self.color.as_ref() {
            "always" => ColorChoice::Always,
            "ansi" => ColorChoice::AlwaysAnsi,
            "auto" => {
                if self.output.is_none() && stdout().is_terminal() {
                    ColorChoice::Auto
                } else {
                    ColorChoice::Never
                }
            }
            _ => ColorChoice::Never,
        };

        let mut tag_color = ColorSpec::new();
        tag_color.set_bold(true);

        let mut occurrence_color = ColorSpec::new();
        occurrence_color.set_bold(true);

        let mut code_color = ColorSpec::new();
        code_color.set_bold(true);

        let mut value_color = ColorSpec::new();

        if let Some(config) = &config.print {
            if let Some(spec) = &config.tag_color {
                tag_color = ColorSpec::try_from(spec)?;
            }
            if let Some(spec) = &config.occurrence_color {
                occurrence_color = ColorSpec::try_from(spec)?;
            }
            if let Some(spec) = &config.code_color {
                code_color = ColorSpec::try_from(spec)?;
            }
            if let Some(spec) = &config.value_color {
                value_color = ColorSpec::try_from(spec)?;
            }
        }

        let mut writer: Box<dyn WriteColor> = match self.output {
            Some(filename) => {
                Box::new(NoColor::new(File::create(filename)?))
            }
            None => Box::new(StandardStream::stdout(choice)),
        };

        let mut progress = Progress::new(self.progress);
        let mut count = 0;

        'outer: for filename in self.filenames {
            let mut reader =
                ReaderBuilder::new().from_path(filename)?;

            while let Some(result) = reader.next() {
                match result {
                    Err(e) => {
                        if e.is_invalid_record() && skip_invalid {
                            progress.invalid();
                            continue;
                        } else {
                            return Err(e.into());
                        }
                    }
                    Ok(record) => {
                        progress.record();

                        for field in record.iter() {
                            writer.set_color(&tag_color)?;
                            write!(writer, "{}", field.tag())?;

                            if let Some(occurrence) = field.occurrence()
                            {
                                writer.set_color(&occurrence_color)?;
                                occurrence.write_to(&mut writer)?;
                            }

                            for subfield in field.subfields() {
                                let code = subfield.code();
                                writer.set_color(&code_color)?;
                                write!(writer, " ${code}")?;

                                let value =
                                    subfield.value().to_str_lossy();
                                let value = translit_maybe(
                                    &value,
                                    self.translit.as_deref(),
                                );

                                writer.set_color(&value_color)?;
                                write!(writer, " {value}")?;
                            }

                            writeln!(writer)?;
                        }

                        writeln!(writer)?;
                        count += 1;

                        if self.limit > 0 && count >= self.limit {
                            break 'outer;
                        }
                    }
                }
            }
        }

        progress.finish();
        writer.flush()?;

        Ok(())
    }
}
