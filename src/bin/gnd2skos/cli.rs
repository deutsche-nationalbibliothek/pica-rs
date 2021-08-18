use clap::{App, Arg};

pub fn build_cli() -> App<'static> {
    App::new("gnd2skos")
        .about("Convert GND records to SKOS.")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .about("skip invalid records"),
        )
        .arg(
            Arg::new("no-relations")
                .long("no-relations")
                .about("Don't add links between concepts.")
                .conflicts_with_all(&["no-broader", "no-related"]),
        )
        .arg(
            Arg::new("no-broader")
                .long("no-broader")
                .about("Don't add broader links between concepts."),
        )
        .arg(
            Arg::new("no-related")
                .long("no-related")
                .about("Don't add related links between concepts."),
        )
        .arg(
            Arg::new("filter")
                .about("A filter expression used for searching.")
                .takes_value(true)
                .long("filter")
                .short('f'),
        )
        .arg(
            Arg::new("output")
                .about("Write output to <file> instead of stdout.")
                .takes_value(true)
                .long("--output")
                .short('o'),
        )
        .arg(Arg::new("filename"))
}
