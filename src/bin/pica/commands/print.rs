use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::str::FromStr;

use clap::Parser;
use pica::{PicaWriter, Reader, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use termcolor::{
    Color, ColorChoice, ColorSpec, StandardStream, WriteColor,
};

use crate::config::Config;
use crate::skip_invalid_flag;
use crate::translit::translit_maybe;
use crate::util::{CliError, CliResult};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub(crate) struct PrintConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) add_spaces: Option<bool>,
    pub(crate) field_color: Option<PrintColorSpec>,
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

#[derive(Parser, Debug)]
pub(crate) struct Print {
    /// Skip invalid records that can't be decoded
    #[arg(short, long)]
    skip_invalid: bool,

    /// Limit the result to first <n> records
    #[arg(long, short, value_name = "n", default_value = "0")]
    limit: usize,

    /// Transliterate output into the selected normalform <NF>
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

    /// Add single space before and after subfield code
    #[arg(long)]
    add_spaces: bool,

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

        let add_spaces = if self.add_spaces {
            true
        } else if let Some(ref config) = config.print {
            config.add_spaces.unwrap_or_default()
        } else {
            false
        };

        if let Some(filename) = self.output {
            let mut writer: Box<dyn PicaWriter> =
                WriterBuilder::new().from_path(filename)?;

            for filename in self.filenames {
                let builder = ReaderBuilder::new()
                    .skip_invalid(skip_invalid)
                    .limit(self.limit);

                let mut reader: Reader<Box<dyn Read>> =
                    match filename.to_str() {
                        Some("-") => {
                            builder.from_reader(Box::new(io::stdin()))
                        }
                        _ => builder.from_path(filename)?,
                    };

                for result in reader.records() {
                    let value = translit_maybe(
                        &format!("{}\n\n", result?),
                        self.translit.as_deref(),
                    );

                    writer.write_all(value.as_bytes())?;
                }
            }

            writer.flush()?;
        } else {
            let color_choice = match self.color.as_ref() {
                "always" => ColorChoice::Always,
                "ansi" => ColorChoice::AlwaysAnsi,
                "auto" => {
                    if atty::is(atty::Stream::Stdout) {
                        ColorChoice::Auto
                    } else {
                        ColorChoice::Never
                    }
                }
                _ => ColorChoice::Never,
            };

            let mut stdout = StandardStream::stdout(color_choice);
            let mut field_color = ColorSpec::new();
            field_color.set_bold(true);

            let mut occurrence_color = ColorSpec::new();
            occurrence_color.set_bold(true);

            let mut code_color = ColorSpec::new();
            code_color.set_bold(true);

            let mut value_color = ColorSpec::new();

            if let Some(config) = &config.print {
                if let Some(spec) = &config.field_color {
                    field_color = ColorSpec::try_from(spec)?;
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

            for filename in self.filenames {
                let builder = ReaderBuilder::new()
                    .skip_invalid(skip_invalid)
                    .limit(self.limit);

                let mut reader: Reader<Box<dyn Read>> =
                    match filename.to_str() {
                        Some("-") => {
                            builder.from_reader(Box::new(io::stdin()))
                        }
                        _ => builder.from_path(filename)?,
                    };

                for result in reader.records() {
                    let record = result?;

                    for field in record.iter() {
                        // TAG
                        stdout.set_color(&field_color)?;
                        write!(stdout, "{}", field.tag())?;

                        // OCCURRENCE
                        if let Some(occurrence) = field.occurrence() {
                            stdout.set_color(&occurrence_color)?;
                            write!(stdout, "/{occurrence}")?;
                        }

                        if !add_spaces {
                            write!(stdout, " ")?;
                        }

                        // SUBFIELDS
                        for subfield in field.iter() {
                            stdout.set_color(&code_color)?;

                            if add_spaces {
                                write!(
                                    stdout,
                                    " ${} ",
                                    subfield.code()
                                )?;
                            } else {
                                write!(stdout, "${}", subfield.code())?;
                            }

                            let mut value: String =
                                subfield.value().to_string();
                            value = translit_maybe(
                                &value.replace('$', "$$"),
                                self.translit.as_deref(),
                            );

                            stdout.set_color(&value_color)?;
                            write!(stdout, "{value}")?;
                        }

                        writeln!(stdout)?;
                    }
                    writeln!(stdout)?;
                }
            }

            stdout.reset()?;
            stdout.flush()?;
        }

        Ok(())
    }
}
