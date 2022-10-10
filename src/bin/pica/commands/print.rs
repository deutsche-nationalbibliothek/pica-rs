use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::str::FromStr;

use clap::Arg;
use pica::{PicaWriter, Reader, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use termcolor::{
    Color, ColorChoice, ColorSpec, StandardStream, WriteColor,
};

use crate::config::Config;
use crate::skip_invalid_flag;
use crate::translit::translit_maybe;
use crate::util::{CliArgs, CliError, CliResult, Command};

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
                    "invalid color '{}'",
                    fg_color_str
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

pub(crate) fn cli() -> Command {
    Command::new("print")
        .about("Print records in human readable format.")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .help("skip invalid records"),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .possible_values(["auto", "always", "ansi", "never"])
                .default_value("auto"),
        )
        .arg(
            Arg::new("add-spaces")
                .long("add-spaces")
                .help("add single space before and after subfield codes."),
        )
        .arg(
            Arg::new("limit")
                .short('l')
                .long("--limit")
                .value_name("n")
                .help("Limit the result to first <n> records."),
        )
        .arg(
            Arg::new("translit")
                .long("--translit")
                .value_name("translit")
                .possible_values(["nfd", "nfkd", "nfc", "nfkc"])
                .help("If present, transliterate output into the selected normalform.")
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .help("Write output to <file> instead of stdout."),
        )
        .arg(
            Arg::new("filenames")
                .help(
                    "Read one or more files in normalized PICA+ format. If the file \
                    ends with .gz the content is automatically decompressed. With no \
                    <filenames>, or when filename is -, read from standard input (stdin).")
                .value_name("filenames")
                .multiple_values(true)
        )
}

pub(crate) fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid =
        skip_invalid_flag!(args, config.print, config.global);

    let add_spaces = if args.is_present("add-spaces") {
        true
    } else if let Some(ref config) = config.print {
        config.add_spaces.unwrap_or_default()
    } else {
        false
    };

    let limit =
        match args.value_of("limit").unwrap_or("0").parse::<usize>() {
            Ok(limit) => limit,
            Err(_) => {
                return Err(CliError::Other(
                    "Invalid limit value, expected unsigned integer."
                        .to_string(),
                ));
            }
        };

    let filenames = args
        .values_of_t::<OsString>("filenames")
        .unwrap_or_else(|_| vec![OsString::from("-")]);

    if let Some(filename) = args.value_of("output") {
        let mut writer: Box<dyn PicaWriter> =
            WriterBuilder::new().from_path(filename)?;

        for filename in filenames {
            let builder = ReaderBuilder::new()
                .skip_invalid(skip_invalid)
                .limit(limit);
            let mut reader: Reader<Box<dyn Read>> = match filename
                .to_str()
            {
                Some("-") => builder.from_reader(Box::new(io::stdin())),
                _ => builder.from_path(filename)?,
            };

            for result in reader.records() {
                let value = translit_maybe(
                    &format!("{}\n\n", result?),
                    args.value_of("translit"),
                );
                writer.write_all(value.as_bytes())?;
            }
        }

        writer.flush()?;
    } else {
        let color_choice = match args.value_of("color").unwrap() {
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

        for filename in filenames {
            let builder = ReaderBuilder::new()
                .skip_invalid(skip_invalid)
                .limit(limit);
            let mut reader: Reader<Box<dyn Read>> = match filename
                .to_str()
            {
                Some("-") => builder.from_reader(Box::new(io::stdin())),
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
                        write!(stdout, "/{}", occurrence)?;
                    }

                    if !add_spaces {
                        write!(stdout, " ")?;
                    }

                    // SUBFIELDS
                    for subfield in field.iter() {
                        stdout.set_color(&code_color)?;

                        if add_spaces {
                            write!(stdout, " ${} ", subfield.code())?;
                        } else {
                            write!(stdout, "${}", subfield.code())?;
                        }

                        let mut value: String =
                            subfield.value().to_string();
                        value = translit_maybe(
                            &value.replace('$', "$$"),
                            args.value_of("translit"),
                        );

                        stdout.set_color(&value_color)?;
                        write!(stdout, "{}", value)?;
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
