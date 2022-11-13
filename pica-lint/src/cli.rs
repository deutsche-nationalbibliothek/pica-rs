use std::ffi::OsString;
use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub(crate) struct Args {
    #[arg(long = "rule-set", short = 'r')]
    pub(crate) rules: Vec<PathBuf>,

    /// Write output to <filename> instead of stdout
    #[arg(short, long, value_name = "filename")]
    pub(crate) output: OsString,

    /// Read one or more files in normalized PICA+ format. If no
    /// filenames where given or a filename is "-", data is read from
    /// standard input (stdin)
    #[arg(default_value = "-", hide_default_value = true)]
    pub(crate) filenames: Vec<OsString>,
}
