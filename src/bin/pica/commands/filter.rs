use crate::util::{App, CliArgs, CliResult};
use clap::{Arg, SubCommand};
use pica::{Path, Record};

pub fn cli() -> App {
    SubCommand::with_name("filter")
        .about("Filter records by whether the given query matches.")
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
        .arg(
            Arg::with_name("query")
                .help("A query expression used for searching.")
                .required(true),
        )
        .arg(Arg::with_name("filename"))
}

pub fn run(_args: &CliArgs) -> CliResult<()> {
    let record: Record = "012A \u{1f}a123\u{1f}a456\u{1e}012A \u{1f}c789\u{1e}"
        .parse::<Record>()
        .unwrap();

    let query = _args.value_of("query").unwrap().parse::<Path>().unwrap();

    // println!("RECORD = {:?}", record);
    // println!("QUERY = {:?}", query);

    let result = record.path(query);
    println!("RESULT = {:?}", result);
    Ok(())
}
