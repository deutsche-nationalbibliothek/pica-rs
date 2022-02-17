use crate::util::{CliArgs, CliResult, Command};
use clap::Arg;
use clap_complete::{generate, Shell};
use std::fs::File;
use std::io::{self, Write};

pub(crate) fn cli() -> Command {
    Command::new("completion")
        .about("Generate a completions file for Bash, Fish or ZSH shell.")
        .arg(
            Arg::new("shell")
                .possible_values(&["fish", "bash", "zsh"])
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .help("Write output to <file> instead of stdout."),
        )
}

pub(crate) fn run(args: &CliArgs, cli: &mut Command) -> CliResult<()> {
    let mut writer: Box<dyn Write> = match args.value_of("output") {
        Some(filename) => Box::new(File::create(filename)?),
        None => Box::new(io::stdout()),
    };

    match args.value_of("shell").unwrap() {
        "bash" => generate(Shell::Bash, cli, "pica", &mut writer),
        "fish" => generate(Shell::Fish, cli, "pica", &mut writer),
        "zsh" => generate(Shell::Zsh, cli, "pica", &mut writer),
        _ => unreachable!(),
    }

    Ok(())
}
