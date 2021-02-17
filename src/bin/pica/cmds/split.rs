use crate::cmds::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::Arg;
use pica::legacy::Record;

use std::fs::create_dir;
use std::io::BufRead;
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
    let ctx = Config::new();
    let skip_invalid = args.is_present("skip-invalid");
    let filename_template = args.value_of("template").unwrap_or("{}.dat");

    let outdir = Path::new(args.value_of("outdir").unwrap());
    if !outdir.exists() {
        create_dir(outdir)?;
    }

    let chunk_size = args
        .value_of("size")
        .unwrap_or("500")
        .parse::<u32>()
        .unwrap();

    if chunk_size == 0 {
        return Err(CliError::Other("chunk size < 1".to_string()));
    }

    let mut chunks: u32 = 0;
    let mut count: u32 = 0;

    let reader = ctx.reader(args.value_of("filename"))?;
    let mut writer = ctx.writer(
        outdir
            .join(filename_template.replace("{}", &chunks.to_string()))
            .to_str(),
    )?;

    for line in reader.lines() {
        let line = line.unwrap();
        if let Ok(_record) = Record::decode(&line) {
            if count % chunk_size == 0 {
                writer.flush()?;

                writer = ctx.writer(
                    outdir
                        .join(
                            filename_template
                                .replace("{}", &chunks.to_string()),
                        )
                        .to_str(),
                )?;
                chunks += 1;
            }

            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
            count += 1;
        } else if !skip_invalid {
            return Err(CliError::Other(format!(
                "could not read record: {}",
                line
            )));
        }
    }

    writer.flush()?;
    Ok(())
}
