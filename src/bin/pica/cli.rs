use clap::{App, AppSettings};

pub fn build_cli() -> App<'static> {
    App::new("pica")
        .about("Tools to work with bibliographic records encoded in Pica+")
        .setting(AppSettings::SubcommandRequired)
        .version(crate_version!())
        .author(crate_authors!())
        .subcommands(crate::commands::subcmds())
}
