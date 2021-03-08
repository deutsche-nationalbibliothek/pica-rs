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
            Arg::new("filter")
                .about("A filter expression used for searching.")
                .takes_value(true)
                .long("filter")
                .short('f'),
        )
        .arg(
            Arg::new("output")
                .about("Write output to <file> instead of stdout.")
                .long("--output")
                .short('o'),
        )
        .arg(Arg::new("filename"))
}
