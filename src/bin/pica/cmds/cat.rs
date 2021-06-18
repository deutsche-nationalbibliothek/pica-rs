use crate::config::Config;
use crate::util::{App, CliArgs, CliResult};
use clap::Arg;
use pica::{PicaWriter, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CatConfig {
    pub skip_invalid: Option<bool>,
    pub gzip: Option<bool>,
}

pub fn cli() -> App {
    App::new("cat")
        .about("Concatenate records from multiple files.")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .about("skip invalid records"),
        )
        .arg(
            Arg::new("gzip")
                .short('g')
                .long("gzip")
                .about("compress output with gzip")
                .requires("output"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .about("Write output to <file> instead of stdout."),
        )
        .arg(Arg::new("filenames").multiple(true).required(true))
}

pub fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = match args.is_present("skip-invalid") {
        false => {
            if let Some(ref cat_config) = config.cat {
                cat_config.skip_invalid.unwrap_or_default()
            } else if let Some(ref global_config) = config.global {
                global_config.skip_invalid.unwrap_or_default()
            } else {
                false
            }
        }
        _ => true,
    };

    let gzip_compression = match args.is_present("gzip") {
        false => {
            if let Some(ref cat_config) = config.cat {
                cat_config.gzip.unwrap_or_default()
            } else {
                false
            }
        }
        _ => true,
    };

    let mut writer: Box<dyn PicaWriter> = WriterBuilder::new()
        .gzip(gzip_compression)
        .from_path_or_stdout(args.value_of("output"))?;

    for filename in args.values_of("filenames").unwrap() {
        let mut reader = ReaderBuilder::new()
            .skip_invalid(skip_invalid)
            .from_path(filename)?;

        for result in reader.byte_records() {
            writer.write_byte_record(&result?)?;
        }
    }

    writer.finish()?;
    Ok(())
}
