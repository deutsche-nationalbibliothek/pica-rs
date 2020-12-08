use crate::commands::Config;
use crate::util::{App, CliArgs, CliResult};
use clap::Arg;
use clap_generate::{
    generate, generators::Bash, generators::Fish, generators::Zsh,
};

pub fn cli() -> App {
    App::new("completion")
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
                .about("Write output to <file> instead of stdout."),
        )
        .about("Generate a completions file for Bash, Fish or ZSH shell.")
}

pub fn run(args: &CliArgs, cli: &mut App) -> CliResult<()> {
    let config = Config::new();
    let mut writer = config.writer(args.value_of("output"))?;

    match args.value_of("shell").unwrap() {
        "bash" => generate::<Bash, _>(cli, "pica", &mut writer),
        "fish" => generate::<Fish, _>(cli, "pica", &mut writer),
        "zsh" => generate::<Zsh, _>(cli, "pica", &mut writer),
        _ => unreachable!(),
    }

    Ok(())
}
