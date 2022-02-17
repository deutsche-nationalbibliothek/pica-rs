use crate::config::Config;
use crate::util::{App, CliArgs, CliResult};
use crate::{gzip_flag, skip_invalid_flag};
use clap::Arg;
use pica::{PicaWriter, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CatConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
}

pub(crate) fn cli() -> App {
    App::new("cat")
        .about("Concatenate records from multiple <files>.")
        .arg(
            Arg::new("skip-invalid")
                .help("Whether to skip invalid records that can't be decoded.")
                .long("skip-invalid")
                .short('s')
        )
        .arg(
            Arg::new("gzip")
                .help("If the --gzip flag is provided the output will be compressed \
                      and written in gzip file format.")
                .long("gzip")
                .short('g')
                .requires("output"),
        )
        .arg(
            Arg::new("output")
                .help("Write output to <filename> instead of stdout. If the file ends with \
                      .gz the file content will be compressed and written in gzip file\
                      format.")
                .value_name("filename")
                .long("--output")
                .short('o')
        )
        .arg(
            Arg::new("filenames")
                .help("Read one or more files in normalized PICA+ format. If the file \
                ends with .gz the content is automatically decompressed.")
                .value_name("files")
                .multiple_values(true)
                .required(true)
        )
}

pub(crate) fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = skip_invalid_flag!(args, config.cat, config.global);
    let gzip_compression = gzip_flag!(args, config.cat);

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
