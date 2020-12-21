use crate::commands::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::Arg;
use pica::Record;
use rand::{thread_rng, Rng};
use std::io::BufRead;

pub fn cli() -> App {
    App::new("sample")
        .about("Selects a random permutation of records")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .about("skip invalid records"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .about("Write output to <file> instead of stdout."),
        )
        .arg(Arg::new("sample-size").required(true))
        .arg(Arg::new("filename"))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let config = Config::new();
    let mut writer = config.writer(args.value_of("output"))?;
    let reader = config.reader(args.value_of("filename"))?;
    let skip_invalid = args.is_present("skip-invalid");

    let sample_size = args.value_of("sample-size").unwrap();
    let n = match sample_size.parse::<usize>() {
        Ok(v) => v,
        Err(_) => {
            return Err(CliError::Other(format!(
                "invalid sample size '{}'. expected usize.",
                sample_size
            )));
        }
    };

    let mut reservoir: Vec<String> = Vec::with_capacity(n);
    let mut rng = thread_rng();

    for (i, line) in reader.lines().enumerate() {
        let line = line.unwrap();

        if let Ok(_record) = Record::decode(&line) {
            if i < n {
                reservoir.push(line);
            } else {
                let j = rng.gen_range(0..i);
                if j < n {
                    reservoir[j] = line;
                }
            }
        } else if !skip_invalid {
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
