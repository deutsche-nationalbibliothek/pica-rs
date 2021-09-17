use clap::{App, AppSettings, Arg};

pub(crate) fn build_cli() -> App<'static> {
    App::new("pica")
        .about("Tools to work with bibliographic records encoded in Pica+")
        .setting(AppSettings::SubcommandRequired)
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
