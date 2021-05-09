use crate::util::{App, CliArgs, CliError, CliResult};
use clap::Arg;
use pica::{ReaderBuilder, WriterBuilder};

use std::fs::create_dir;
use std::path::Path;

pub fn cli() -> App {
    App::new("split")
        .about("Splits a stream of records into chunks.")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .about("skip invalid records"),
        )
        .arg(
            Arg::new("outdir")
                .short('o')
                .long("--outdir")
                .value_name("outdir")
                .default_value("."),
        )
        .arg(
            Arg::new("template")
                .short('t')
                .long("--template")
                .value_name("template")
                .default_value("{}.dat"),
        )
        .arg(Arg::new("size").required(true))
        .arg(Arg::new("filename"))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let mut reader = ReaderBuilder::new()
        .skip_invalid(args.is_present("skip-invalid"))
        .from_path_or_stdin(args.value_of("filename"))?;

    let filename_template = args.value_of("template").unwrap_or("{}.dat");

    let outdir = Path::new(args.value_of("outdir").unwrap());
    if !outdir.exists() {
        create_dir(outdir)?;
    }

    let chunk_size = args.value_of("size").unwrap_or("500");
    let chunk_size = match chunk_size.parse::<u32>() {
        Ok(size) => size,
        Err(_) => {
            return Err(CliError::Other("invalid chunk size".to_string()))
        }
    };

    if chunk_size == 0 {
        return Err(CliError::Other("chunk size < 1".to_string()));
    }

    let mut chunks: u32 = 0;

    let mut writer = WriterBuilder::new().from_path(
        outdir
            .join(filename_template.replace("{}", &chunks.to_string()))
            .to_str()
            .unwrap(),
    )?;

    for (count, result) in reader.byte_records().enumerate() {
        let record = result?;

        if count > 0 && count as u32 % chunk_size == 0 {
            writer.flush()?;
            chunks += 1;

            writer = WriterBuilder::new().from_path(
                outdir
                    .join(filename_template.replace("{}", &chunks.to_string()))
                    .to_str()
                    .unwrap(),
            )?;
        }

        writer.write_byte_record(&record)?;
    }

    writer.flush()?;
    Ok(())
}
