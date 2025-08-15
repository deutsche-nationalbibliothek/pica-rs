use std::path::PathBuf;

use clap::{Parser, Subcommand};
use pica_record::path::Path;

use crate::commands::*;

/// pica is a fast command-line tool to process bibliographic records
/// encoded in PICA+.
#[derive(Debug, Parser)]
#[clap(version, author, infer_subcommands = true, max_term_width = 72)]
#[command(name = "pica", version, about, long_about = None)]
pub(crate) struct Args {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    pub(crate) config: Option<PathBuf>,

    #[command(subcommand)]
    pub(crate) cmd: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    Check(Check),
    Completions(Completions),
    Concat(Concat),
    Config(Config),
    Convert(Convert),
    Count(Count),
    Describe(Describe),
    Explode(Explode),
    Filter(Filter),
    Frequency(Frequency),
    Hash(Hash),
    Invalid(Invalid),
    Partition(Partition),
    Print(Print),
    Sample(Sample),
    Select(Select),
    Slice(Slice),
    Split(Split),
}

#[derive(Debug, clap::Args)]
pub(crate) struct FilterOpts {
    /// Skip invalid records that can't be decoded
    #[arg(short, long)]
    pub(crate) skip_invalid: bool,

    /// Limit the result to first N records
    ///
    /// # Note
    ///
    /// A limit value `0` means no limit.
    #[arg(long, short, value_name = "N", default_value = "0")]
    pub(crate) limit: usize,

    /// Ignore records which are *not* explicitly listed in one of the
    /// given allow-lists.
    ///
    /// An allow-list must be a CSV, TSV or Apache Arrow file. By
    /// default the column `ppn` or `idn` is used to get the values
    /// of the allow list. These values are compared against the PPN
    /// (003@.0) of record.
    ///
    /// The column name can be changed using the `--filter-set-column`
    /// option and the path to the comparison values can be changed
    /// with option `--filter-set-source`.
    ///
    /// # Note
    ///
    /// If the allow list is empty, all records are blocked. With more
    /// than one allow list, the filter set is made up of all partial
    /// lists. lists.
    #[arg(long = "allow-list", short = 'A')]
    pub(crate) allow: Vec<PathBuf>,

    /// Ignore records which are explicitly listed in one of the
    /// given deny-lists.
    ///
    /// A deny-list must be a CSV, TSV or Apache Arrow file. By
    /// default the column `ppn` or `idn` is used to get the values
    /// of the allow list. These values are compared against the PPN
    /// (003@.0) of record.
    ///
    /// The column name can be changed using the `--filter-set-column`
    /// option and the path to the comparison values can be changed
    /// with option `--filter-set-source`.
    ///
    /// # Note
    ///
    /// With more than one deny list, the filter set is made up of all
    /// partial lists.
    #[arg(long = "deny-list", short = 'D')]
    pub(crate) deny: Vec<PathBuf>,

    /// Defines the column name of an allow-list or a deny-list. By
    /// default, the column `ppn` is used or, if this is not
    /// available, the column `idn` is used.
    #[arg(long, value_name = "COLUMN")]
    pub(crate) filter_set_column: Option<String>,

    /// Defines an optional path to the comparison values of the
    /// record. If no path is specified, a comparison with the PPN in
    /// field 003@.0 is assumed.
    #[arg(long, value_name = "PATH")]
    pub(crate) filter_set_source: Option<Path>,
}
