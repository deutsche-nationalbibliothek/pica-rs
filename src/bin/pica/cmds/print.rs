use crate::config::Config;
use crate::skip_invalid_flag;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::Arg;
use pica::{PicaWriter, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct PrintConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) add_spaces: Option<bool>,
}

pub(crate) fn cli() -> App {
    App::new("print")
        .about("Print records in human readable format.")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .about("skip invalid records"),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .possible_values(&["auto", "always", "ansi", "never"])
                .default_value("auto"),
        )
        .arg(
            Arg::new("add-spaces")
                .long("add-spaces")
                .about("add single space before and after subfield codes."),
        )
        .arg(
            Arg::new("limit")
                .short('l')
                .long("--limit")
                .value_name("n")
                .about("Limit the result to first <n> records."),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .about("Write output to <file> instead of stdout."),
        )
        .arg(Arg::new("filename"))
}

pub(crate) fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = skip_invalid_flag!(args, config.print, config.global);

    let add_spaces = if args.is_present("add-spaces") {
        true
    } else if let Some(ref config) = config.print {
        config.add_spaces.unwrap_or_default()
    } else {
        false
    };

    let limit = match args.value_of("limit").unwrap_or("0").parse::<usize>() {
        Ok(limit) => limit,
        Err(_) => {
            return Err(CliError::Other(
                "Invalid limit value, expected unsigned integer.".to_string(),
            ));
        }
    };

    let mut reader = ReaderBuilder::new()
        .skip_invalid(skip_invalid)
        .limit(limit)
        .from_path_or_stdin(args.value_of("filename"))?;

    if let Some(filename) = args.value_of("output") {
        let mut writer: Box<dyn PicaWriter> =
            WriterBuilder::new().from_path(filename)?;

        for result in reader.records() {
            writer.write_all(format!("{}\n\n", result?).as_bytes())?;
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

        for result in reader.records() {
            let record = result?;

            for field in record.iter() {
                stdout
                    .set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))?;

                // TAG
                write!(&mut stdout, "{}", field.tag())?;

                // OCCURRENCE
                if let Some(occurrence) = field.occurrence() {
                    write!(&mut stdout, "{}", occurrence)?;
                }

                if !add_spaces {
                    write!(&mut stdout, " ")?;
                }

                // SUBFIELDS
                for subfield in field.iter() {
                    stdout
                        .set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;

                    if add_spaces {
                        write!(&mut stdout, " ${} ", subfield.code())?;
                    } else {
                        write!(&mut stdout, "${}", subfield.code())?;
                    }

                    stdout.set_color(
                        ColorSpec::new().set_fg(Some(Color::White)),
                    )?;

                    let mut value: String = subfield.value().to_string();
                    value = value.replace("$", "$$");

                    write!(&mut stdout, "{}", value)?;
                }

                writeln!(&mut stdout)?;
            }
            writeln!(&mut stdout)?;
        }

        stdout.flush()?;
    }

    Ok(())
}
