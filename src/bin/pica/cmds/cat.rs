use crate::util::{App, CliArgs, CliResult};
use crate::Config;
use clap::Arg;
use pica::{PicaWriter, ReaderBuilder, WriterBuilder};

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
                .about("compress output with gzip"),
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
        false => config
            .get_bool("cat", "skip-invalid", true)
            .unwrap_or_default(),
        _ => true,
    };

    let gzip_compress = match args.is_present("gzip") {
        false => config.get_bool("cat", "gzip", false).unwrap_or_default(),
        _ => true,
    };

    let mut writer: Box<dyn PicaWriter> = WriterBuilder::new()
        .gzip(gzip_compress)
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
