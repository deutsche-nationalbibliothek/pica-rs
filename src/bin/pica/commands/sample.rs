use crate::util::{App, CliArgs, CliError, CliResult};
use clap::{Arg, SubCommand};
use pica::Record;
use rand::{thread_rng, Rng};
use std::boxed::Box;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::str::FromStr;

pub fn cli() -> App {
    SubCommand::with_name("sample")
        .about("Selects a random permutation of records")
        .arg(
            Arg::with_name("skip-invalid")
                .short("s")
                .long("skip-invalid")
                .help("skip invalid records"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("--output")
                .value_name("file")
                .help("Write output to <file> instead of stdout."),
        )
        .arg(Arg::with_name("sample-size").required(true))
        .arg(Arg::with_name("filename"))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let reader: Box<dyn BufRead> = match args.value_of("filename") {
        None => Box::new(BufReader::new(io::stdin())),
        Some(filename) => Box::new(BufReader::new(File::open(filename)?)),
    };

    let mut writer: Box<dyn Write> = match args.value_of("output") {
        None => Box::new(io::stdout()),
        Some(filename) => Box::new(File::create(filename)?),
    };

    let sample_size =
        match args.value_of("sample-size").unwrap().parse::<usize>() {
            Ok(v) => v,
            Err(_) => {
                return Err(CliError::Other("invalid sample size".to_string()))
            }
        };

    let mut reservoir: Vec<String> = Vec::with_capacity(sample_size);
    let mut rng = thread_rng();

    for (i, line) in reader.lines().enumerate() {
        let line = line.unwrap();

        if let Ok(_record) = Record::from_str(&line) {
            if i < sample_size {
                reservoir.push(line);
            } else {
                let j = rng.gen_range(0, i);
                if j < sample_size {
                    reservoir[j] = line;
                }
            }
        } else if !args.is_present("skip-invalid") {
            return Err(CliError::Other(format!(
                "could not read record: {}",
                line
            )));
        }
    }

    for line in reservoir {
        writer.write_all(line.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    writer.flush()?;
    Ok(())
}
