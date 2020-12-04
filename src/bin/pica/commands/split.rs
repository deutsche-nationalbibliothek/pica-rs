use crate::commands::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::{Arg, SubCommand};
use pica::Record;

use std::fs::create_dir;
use std::io::BufRead;
use std::path::Path;

pub fn cli() -> App {
    SubCommand::with_name("split")
        .about("Splits a stream of records into chunks.")
        .arg(
            Arg::with_name("skip-invalid")
                .short("s")
                .long("skip-invalid")
                .help("skip invalid records"),
        )
        .arg(
            Arg::with_name("outdir")
                .short("o")
                .long("--outdir")
                .value_name("outdir")
                .default_value("."),
        )
        .arg(
            Arg::with_name("template")
                .short("t")
                .long("--template")
                .value_name("template")
                .default_value("{}.dat"),
        )
        .arg(Arg::with_name("size").required(true))
        .arg(Arg::with_name("filenames").multiple(true).required(true))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let config = Config::new();

    let filename_template = args.value_of("filename").unwrap_or("{}.dat");
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

    let mut writer = config.writer(
        outdir
            .join(filename_template.replace("{}", &chunks.to_string()))
            .to_str(),
    )?;

    for filename in args.values_of("filenames").unwrap() {
        let reader = config.reader(Some(filename))?;

        for line in reader.lines() {
            let line = line.unwrap();
            if let Ok(_record) = Record::decode(&line) {
                if count % chunk_size == 0 {
                    writer.flush()?;

                    writer = config.writer(
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
            }
        }
    }

    // let mut writer = config.writer(args.value_of("output"))?;
    // let skip_invalid = args.is_present("skip-invalid");

    // let mut count: u32 = 0;

    // for filename in args.values_of("filenames").unwrap() {
    //     let reader = config.reader(Some(filename))?;

    //     for line in reader.lines() {
    //         let line = line.unwrap();
    //         if let Ok(_record) = Record::decode(&line) {
    //             println!("count = {:?}", count);
    //             count += 1;

    //             //     writer.write_all(line.as_bytes())?;
    //             //     writer.write_all(b"\n")?;
    //         } else if !skip_invalid {
    //             return Err(CliError::Other(format!(
    //                 "could not read record: {}",
    //                 line
    //             )));
    //         }
    //     }
    // }

    writer.flush()?;
    Ok(())
}
