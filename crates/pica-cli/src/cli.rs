use std::path::PathBuf;

use clap::{Parser, Subcommand, value_parser};
use pica_record::prelude::*;

use crate::commands::*;
use crate::error::CliError;
use crate::prelude::{NormalizationForm, translit};

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

    /// Limit the result to first N records (a limit value `0` means no
    /// limit).
    #[arg(long, short, value_name = "N", default_value = "0")]
    pub(crate) limit: usize,

    /// When this flag is set, comparison operations will be search
    /// case insensitive
    #[arg(long, short)]
    pub(crate) ignore_case: bool,

    /// The minimum score for string similarity comparisons (0 <= score
    /// < 100).
    #[arg(long, value_parser = value_parser!(u8).range(0..100),
          default_value = "75")]
    pub(crate) strsim_threshold: u8,

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

    /// A filter expression used for searching
    #[arg(long = "where", value_name = "FILTER")]
    r#where: Option<String>,

    /// Connects the where clause with additional expressions using the
    /// logical AND-operator (conjunction)
    ///
    /// This option can't be combined with `--or`.
    #[arg(long, requires = "where", conflicts_with = "or")]
    and: Vec<String>,

    /// Connects the where clause with additional expressions using the
    /// logical OR-operator (disjunction)
    ///
    /// This option can't be combined with `--and` or `--not`.
    #[arg(long, requires = "where", conflicts_with_all = ["and", "not"])]
    or: Vec<String>,

    /// Connects the where clause with additional expressions using the
    /// logical NOT-operator (negation)
    ///
    /// This option can't be combined with `--and` or `--or`.
    #[arg(long, requires = "where", conflicts_with = "or")]
    not: Vec<String>,
}

impl FilterOpts {
    pub(crate) fn matcher(
        &self,
        nf: Option<NormalizationForm>,
        predicate: Option<String>,
    ) -> Result<Option<RecordMatcher>, CliError> {
        let filter = predicate.or(self.r#where.clone());

        Ok(if let Some(ref matcher) = filter {
            Some(
                RecordMatcherBuilder::with_transform(
                    matcher.clone(),
                    translit(nf),
                )?
                .and(self.and.clone())?
                .or(self.or.clone())?
                .not(self.not.clone())?
                .build(),
            )
        } else {
            None
        })
    }
}

impl From<&FilterOpts> for MatcherOptions {
    fn from(opts: &FilterOpts) -> Self {
        MatcherOptions::new()
            .strsim_threshold(opts.strsim_threshold as f64 / 100f64)
            .case_ignore(opts.ignore_case)
    }
}
