use std::fs::File;
use std::io;
use std::path::Path;

use bstr::{BStr, BString};
use hashbrown::HashSet;
use polars::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum FilterSetError {
    #[error(transparent)]
    Polars(#[from] PolarsError),
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error("{0}")]
    Other(String),
}

pub(crate) struct FilterSet {
    allow: HashSet<BString>,
    deny: HashSet<BString>,
}

impl FilterSet {
    pub(crate) fn new<P: AsRef<Path>>(
        allow: Vec<P>,
        deny: Vec<P>,
    ) -> Result<Self, FilterSetError> {
        Ok(Self {
            allow: read_filter_set(allow)?,
            deny: read_filter_set(deny)?,
        })
    }

    #[inline]
    pub(crate) fn check(&self, ppn: &BStr) -> bool {
        if self.allow.is_empty() && self.deny.is_empty() {
            return true;
        }

        if !self.allow.is_empty() && !self.allow.contains(ppn) {
            false
        } else {
            !self.deny.contains(ppn)
        }
    }
}

fn read_filter_set<P: AsRef<Path>>(
    paths: Vec<P>,
) -> Result<HashSet<BString>, FilterSetError> {
    let mut set = HashSet::new();

    for path in paths.iter() {
        let df = read_df(path)?;
        let column =
            df.column("ppn").or(df.column("idn")).map_err(|_| {
                FilterSetError::Other(format!(
                    "Missing a column `ppn` or `idn` in file {}. ",
                    path.as_ref().display()
                ))
            })?;

        set.extend(
            column
                .cast(&DataType::String)?
                .str()?
                .iter()
                .filter_map(|idn| idn.map(BString::from)),
        );
    }

    Ok(set)
}

fn read_df<P: AsRef<Path>>(
    path: P,
) -> Result<DataFrame, FilterSetError> {
    let path = path.as_ref().to_path_buf();
    let path_str = path.to_str().unwrap_or_default();

    if path_str.ends_with("ipc") || path_str.ends_with("arrow") {
        Ok(IpcReader::new(File::open(path)?)
            .memory_mapped(None)
            .finish()?)
    } else if path_str.ends_with("tsv") || path_str.ends_with("tsv.gz")
    {
        Ok(CsvReadOptions::default()
            .with_has_header(true)
            .with_infer_schema_length(Some(0))
            .with_parse_options(
                CsvParseOptions::default().with_separator(b'\t'),
            )
            .try_into_reader_with_file_path(Some(path))?
            .finish()?)
    } else {
        Ok(CsvReadOptions::default()
            .with_has_header(true)
            .with_infer_schema_length(Some(0))
            .try_into_reader_with_file_path(Some(path))?
            .finish()?)
    }
}
