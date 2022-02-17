use clap::{crate_version, Arg, Command};

pub(crate) fn build_cli() -> Command<'static> {
    Command::new("pica")
        .about("Tools to work with bibliographic records encoded in Pica+")
        .subcommand_required(true)
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .takes_value(true)
                .value_name("filename"),
        )
        .subcommands(crate::cmds::subcmds())
}
